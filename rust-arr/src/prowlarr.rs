//! Read-only Prowlarr indexer discovery and explicit search.
use super::*;
use std::collections::HashMap;

const MAX_SEARCH_QUERY_CHARS: usize = 120;
const MAX_SEARCH_RESULTS: usize = 50;
const MAX_RESPONSE_BYTES: usize = 2 * 1024 * 1024;

type ApiResult<T> = Result<Json<T>, (StatusCode, Json<ApiError>)>;

fn failure(status: StatusCode, error: &'static str) -> (StatusCode, Json<ApiError>) {
    (status, Json(ApiError { error }))
}

async fn vault(state: &AppState) -> Result<secure_config::AgeVault, (StatusCode, Json<ApiError>)> {
    state.vault.read().await.clone().ok_or(failure(
        StatusCode::LOCKED,
        "Unlock the age-encrypted configuration first",
    ))
}

fn connection(
    vault: &secure_config::AgeVault,
) -> Result<Option<secure_config::ArrConnection>, (StatusCode, Json<ApiError>)> {
    Ok(vault
        .load()
        .map_err(|_| failure(StatusCode::INTERNAL_SERVER_ERROR, "Could not read encrypted settings"))?
        .and_then(|config| {
            config
                .additional_services
                .iter()
                .find(|service| service.service_type == "prowlarr")
                .map(|service| secure_config::ArrConnection {
                    url: service.url.clone(),
                    api_key: service.api_key.clone(),
                })
        }))
}

fn make_client(
    url: String,
    api_key: String,
) -> Result<ArrClient, (StatusCode, Json<ApiError>)> {
    ArrClient::new(url, api_key)
        .map_err(|_| failure(StatusCode::BAD_REQUEST, "Invalid Prowlarr settings"))
}

async fn configured_client(
    state: &AppState,
) -> Result<ArrClient, (StatusCode, Json<ApiError>)> {
    let vault = vault(state).await?;
    let config = connection(&vault)?.ok_or(failure(
        StatusCode::SERVICE_UNAVAILABLE,
        "Prowlarr is not configured",
    ))?;
    make_client(config.url.clone(), config.api_key.clone())
}

async fn status(client: &ArrClient) -> Result<ProwlarrStatus, reqwest::Error> {
    client
        .http
        .get(format!("{}/api/v1/system/status", client.base_url))
        .header("X-Api-Key", client.api_key.clone())
        .header(ACCEPT, "application/json")
        .send()
        .await?
        .error_for_status()?
        .json()
        .await
}

async fn bounded_json(response: reqwest::Response) -> Result<serde_json::Value, ()> {
    if !response.status().is_success()
        || response
            .content_length()
            .is_some_and(|length| length > MAX_RESPONSE_BYTES as u64)
    {
        return Err(());
    }
    let mut response = response;
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(|_| ())? {
        if bytes.len().saturating_add(chunk.len()) > MAX_RESPONSE_BYTES {
            return Err(());
        }
        bytes.extend_from_slice(&chunk);
    }
    serde_json::from_slice(&bytes).map_err(|_| ())
}

async fn indexer_data(client: &ArrClient) -> Result<(serde_json::Value, serde_json::Value), ()> {
    let indexers = client
        .http
        .get(format!("{}/api/v1/indexer", client.base_url))
        .header("X-Api-Key", client.api_key.clone())
        .header(ACCEPT, "application/json")
        .send()
        .await
        .map_err(|_| ())?;
    let statuses = client
        .http
        .get(format!("{}/api/v1/indexerstatus", client.base_url))
        .header("X-Api-Key", client.api_key.clone())
        .header(ACCEPT, "application/json")
        .send()
        .await
        .map_err(|_| ())?;
    tokio::try_join!(bounded_json(indexers), bounded_json(statuses))
        .map_err(|_| ())
}

fn normalize_indexers(
    indexers: serde_json::Value,
    statuses: serde_json::Value,
) -> Result<Vec<serde_json::Value>, ()> {
    let indexers = indexers.as_array().ok_or(())?;
    let statuses = statuses.as_array().ok_or(())?;
    let status_by_indexer: HashMap<i64, &serde_json::Value> = statuses
        .iter()
        .filter_map(|status| {
            Some((
                status.get("indexerId")?.as_i64()?,
                status,
            ))
        })
        .collect();
    Ok(indexers
        .iter()
        .filter_map(|indexer| {
            let id = indexer.get("id")?.as_i64()?;
            let enabled = indexer.get("enable").and_then(serde_json::Value::as_bool)?;
            let status = status_by_indexer.get(&id).copied();
            let disabled_until = status
                .and_then(|value| value.get("disabledTill"))
                .filter(|value| !value.is_null())
                .and_then(serde_json::Value::as_str);
            Some(serde_json::json!({
                "id": id,
                "name": indexer.get("name").and_then(serde_json::Value::as_str).unwrap_or("Unknown indexer"),
                "enabled": enabled,
                "status": if !enabled { "disabled" } else if disabled_until.is_some() { "temporarily-disabled" } else { "enabled" },
                "disabledUntil": disabled_until,
                "protocol": indexer.get("protocol").and_then(serde_json::Value::as_str),
                "supportsSearch": indexer.get("supportsSearch").and_then(serde_json::Value::as_bool).unwrap_or(false)
            }))
        })
        .collect())
}

fn bounded_text(value: Option<&serde_json::Value>, max_chars: usize) -> Option<String> {
    value
        .and_then(serde_json::Value::as_str)
        .filter(|text| !text.is_empty())
        .map(|text| text.chars().take(max_chars).collect())
}

fn normalize_search_results(
    results: serde_json::Value,
    indexers: &[serde_json::Value],
) -> Result<Vec<serde_json::Value>, ()> {
    let results = results.as_array().ok_or(())?;
    let names: HashMap<i64, &str> = indexers
        .iter()
        .filter_map(|indexer| {
            Some((
                indexer.get("id")?.as_i64()?,
                indexer.get("name")?.as_str()?,
            ))
        })
        .collect();
    Ok(results
        .iter()
        .take(MAX_SEARCH_RESULTS)
        .filter_map(|result| {
            let guid = bounded_text(result.get("guid"), 1024)?;
            let title = bounded_text(result.get("title"), 500)?;
            let indexer_id = result.get("indexerId").and_then(serde_json::Value::as_i64);
            let indexer_name = indexer_id
                .and_then(|id| names.get(&id).copied())
                .or_else(|| result.get("indexer").and_then(serde_json::Value::as_str));
            let categories = result
                .get("categories")
                .and_then(serde_json::Value::as_array)
                .map(|categories| {
                    categories
                        .iter()
                        .filter_map(|category| {
                            category
                                .as_str()
                                .or_else(|| category.get("name").and_then(serde_json::Value::as_str))
                        })
                        .take(12)
                        .map(|category| category.chars().take(64).collect::<String>())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            Some(serde_json::json!({
                "id": guid,
                "title": title,
                "indexerId": indexer_id,
                "indexerName": indexer_name,
                "protocol": bounded_text(result.get("protocol"), 32),
                "sizeBytes": result.get("size").and_then(serde_json::Value::as_u64),
                "publishedAt": bounded_text(result.get("publishDate"), 64),
                "categories": categories
            }))
        })
        .collect())
}

fn validated_search_query(query: &str) -> Result<&str, ()> {
    let query = query.trim();
    if query.is_empty() || query.chars().count() > MAX_SEARCH_QUERY_CHARS {
        return Err(());
    }
    Ok(query)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct ProwlarrSettingsRequest {
    url: String,
    api_key: Option<String>,
}

impl Drop for ProwlarrSettingsRequest {
    fn drop(&mut self) {
        self.api_key.zeroize();
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct SearchRequest {
    query: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProwlarrStatus {
    app_name: String,
    version: String,
}

pub(super) async fn settings(State(state): State<Arc<AppState>>) -> ApiResult<ArrSettingsInfo> {
    let config = connection(&vault(&state).await?)?;
    Ok(Json(ArrSettingsInfo {
        configured: config.is_some(),
        persisted: config.is_some(),
        url: config.map(|config| config.url.clone()),
        has_ui_auth: false,
    }))
}

pub(super) async fn test_connection(
    Json(mut request): Json<ProwlarrSettingsRequest>,
) -> ApiResult<serde_json::Value> {
    let api_key = request
        .api_key
        .take()
        .filter(|key| !key.trim().is_empty())
        .ok_or(failure(
            StatusCode::BAD_REQUEST,
            "Enter a Prowlarr API key to test the connection",
        ))?;
    let client = make_client(request.url.clone(), api_key)?;
    let status = status(&client).await.map_err(|_| {
        failure(
            StatusCode::BAD_GATEWAY,
            "Could not connect to Prowlarr; check the URL and API key",
        )
    })?;
    if !status.app_name.eq_ignore_ascii_case("prowlarr") {
        return Err(failure(
            StatusCode::BAD_GATEWAY,
            "The service did not identify itself as Prowlarr",
        ));
    }
    Ok(Json(
        serde_json::json!({ "ok": true, "version": status.version }),
    ))
}

pub(super) async fn save(
    State(state): State<Arc<AppState>>,
    Json(mut request): Json<ProwlarrSettingsRequest>,
) -> ApiResult<ArrSettingsInfo> {
    let vault = vault(&state).await?;
    let api_key = request
        .api_key
        .take()
        .filter(|key| !key.trim().is_empty())
        .or(connection(&vault)?.map(|config| config.api_key.clone()))
        .ok_or(failure(
            StatusCode::BAD_REQUEST,
            "Enter a Prowlarr API key for the first save",
        ))?;
    let client = make_client(request.url.clone(), api_key)?;
    let status = status(&client).await.map_err(|_| {
        failure(
            StatusCode::BAD_GATEWAY,
            "Could not connect to Prowlarr; check the URL and API key",
        )
    })?;
    if !status.app_name.eq_ignore_ascii_case("prowlarr") {
        return Err(failure(
            StatusCode::BAD_GATEWAY,
            "The service did not identify itself as Prowlarr",
        ));
    }
    let api_key = client
        .api_key
        .to_str()
        .map_err(|_| failure(StatusCode::BAD_REQUEST, "Invalid API key"))?;
    vault
        .save_prowlarr(Some(secure_config::ArrConnection {
            url: client.base_url.clone(),
            api_key: api_key.to_owned(),
        }))
        .map_err(|_| failure(StatusCode::INTERNAL_SERVER_ERROR, "Could not save encrypted Prowlarr settings"))?;
    Ok(Json(ArrSettingsInfo {
        configured: true,
        persisted: true,
        url: Some(client.base_url),
        has_ui_auth: false,
    }))
}

pub(super) async fn clear(State(state): State<Arc<AppState>>) -> ApiResult<ArrSettingsInfo> {
    vault(&state)
        .await?
        .save_prowlarr(None)
        .map_err(|_| failure(StatusCode::INTERNAL_SERVER_ERROR, "Could not clear Prowlarr settings"))?;
    Ok(Json(ArrSettingsInfo {
        configured: false,
        persisted: false,
        url: None,
        has_ui_auth: false,
    }))
}

pub(super) async fn indexers(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Vec<serde_json::Value>> {
    let client = configured_client(&state).await?;
    let (indexers, statuses) = indexer_data(&client).await.map_err(|_| {
        failure(
            StatusCode::BAD_GATEWAY,
            "Prowlarr indexer status request failed",
        )
    })?;
    normalize_indexers(indexers, statuses)
        .map(Json)
        .map_err(|_| failure(StatusCode::BAD_GATEWAY, "Prowlarr returned invalid indexer data"))
}

pub(super) async fn search(
    State(state): State<Arc<AppState>>,
    Json(request): Json<SearchRequest>,
) -> ApiResult<Vec<serde_json::Value>> {
    let query = validated_search_query(&request.query).map_err(|_| {
        failure(
            StatusCode::BAD_REQUEST,
            "Enter a search term of 1 to 120 characters",
        )
    })?;
    let client = configured_client(&state).await?;
    let response = client
        .http
        .get(format!("{}/api/v1/search", client.base_url))
        .header("X-Api-Key", client.api_key.clone())
        .header(ACCEPT, "application/json")
        .query(&[("query", query), ("type", "search"), ("limit", "50")])
        .send()
        .await
        .map_err(|_| failure(StatusCode::BAD_GATEWAY, "Prowlarr search request failed"))?;
    let results = bounded_json(response)
        .await
        .map_err(|_| failure(StatusCode::BAD_GATEWAY, "Prowlarr returned invalid or oversized search results"))?;
    let (indexers, _) = indexer_data(&client).await.map_err(|_| {
        failure(
            StatusCode::BAD_GATEWAY,
            "Prowlarr indexer provenance could not be loaded",
        )
    })?;
    let indexers = indexers
        .as_array()
        .ok_or(failure(StatusCode::BAD_GATEWAY, "Prowlarr returned invalid indexer data"))?;
    normalize_search_results(results, indexers)
        .map(Json)
        .map_err(|_| failure(StatusCode::BAD_GATEWAY, "Prowlarr returned invalid search results"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn indexer_statuses_are_normalized_without_upstream_error_text() {
        let indexers = serde_json::json!([
            {"id": 7, "name": "Fixture", "enable": true, "protocol": "torrent", "supportsSearch": true},
            {"id": 8, "name": "Disabled", "enable": false, "protocol": "usenet"}
        ]);
        let statuses = serde_json::json!([
            {"indexerId": 7, "disabledTill": "2026-11-03T20:00:00Z", "mostRecentFailure": "private error"}
        ]);
        let normalized = normalize_indexers(indexers, statuses).unwrap();
        assert_eq!(normalized[0]["id"], 7);
        assert_eq!(normalized[0]["status"], "temporarily-disabled");
        assert_eq!(normalized[1]["status"], "disabled");
        assert!(normalized[0].get("mostRecentFailure").is_none());
    }

    #[test]
    fn search_results_are_bounded_and_never_expose_download_urls() {
        let results = serde_json::json!([{
            "guid": "stable-result-id", "title": "Fixture release", "indexerId": 7,
            "protocol": "torrent", "size": 1234, "publishDate": "2026-11-03T00:00:00Z",
            "categories": [{"id": 2000, "name": "Movies"}],
            "downloadUrl": "https://private.example/download?apikey=secret",
            "magnetUrl": "magnet:?xt=private"
        }]);
        let indexers = vec![serde_json::json!({"id": 7, "name": "Fixture indexer"})];
        let normalized = normalize_search_results(results, &indexers).unwrap();
        assert_eq!(normalized.len(), 1);
        assert_eq!(normalized[0]["id"], "stable-result-id");
        assert_eq!(normalized[0]["indexerName"], "Fixture indexer");
        assert_eq!(normalized[0]["sizeBytes"], 1234);
        assert_eq!(normalized[0]["categories"], serde_json::json!(["Movies"]));
        assert!(normalized[0].get("downloadUrl").is_none());
        assert!(normalized[0].get("magnetUrl").is_none());
    }

    #[test]
    fn search_query_is_trimmed_and_length_limited() {
        assert_eq!(validated_search_query("  dinosaur  ").unwrap(), "dinosaur");
        assert!(validated_search_query("  ").is_err());
        assert!(validated_search_query(&"x".repeat(MAX_SEARCH_QUERY_CHARS + 1)).is_err());
        assert!(validated_search_query(&"x".repeat(MAX_SEARCH_QUERY_CHARS)).is_ok());
    }
}
