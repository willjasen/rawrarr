//! Read-only Lidarr integration using the shared Arr transport and subscriber vault.
use super::*;

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
                .find(|service| service.service_type == "lidarr")
                .map(|service| secure_config::ArrConnection {
                    url: service.url.clone(),
                    api_key: service.api_key.clone(),
                })
        }))
}

async fn configured_client(
    state: &AppState,
) -> Result<ArrClient, (StatusCode, Json<ApiError>)> {
    let vault = vault(state).await?;
    let config = connection(&vault)?.ok_or(failure(
        StatusCode::SERVICE_UNAVAILABLE,
        "Lidarr is not configured",
    ))?;
    ArrClient::new(config.url.clone(), config.api_key.clone())
        .map_err(|_| failure(StatusCode::BAD_REQUEST, "Invalid Lidarr settings"))
}

fn validate_status(status: &RadarrStatus) -> Result<(), (StatusCode, Json<ApiError>)> {
    if status.app_name.eq_ignore_ascii_case("lidarr") {
        Ok(())
    } else {
        Err(failure(
            StatusCode::BAD_GATEWAY,
            "The service did not identify itself as Lidarr",
        ))
    }
}

async fn fetch_status(client: &ArrClient) -> Result<RadarrStatus, reqwest::Error> {
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

pub(super) async fn status(
    State(state): State<Arc<AppState>>,
) -> ApiResult<RadarrStatus> {
    let client = configured_client(&state).await?;
    let status = fetch_status(&client)
        .await
        .map_err(|_| failure(StatusCode::BAD_GATEWAY, "Lidarr status request failed"))?;
    validate_status(&status)?;
    Ok(Json(status))
}

pub(super) async fn settings(
    State(state): State<Arc<AppState>>,
) -> ApiResult<ArrSettingsInfo> {
    let config = connection(&vault(&state).await?)?;
    Ok(Json(ArrSettingsInfo {
        configured: config.is_some(),
        persisted: config.is_some(),
        url: config.map(|connection| connection.url.clone()),
        has_ui_auth: false,
    }))
}

pub(super) async fn test_connection(
    Json(mut request): Json<RadarrSettingsRequest>,
) -> ApiResult<serde_json::Value> {
    let key = request
        .api_key
        .take()
        .filter(|key| !key.trim().is_empty())
        .ok_or(failure(
            StatusCode::BAD_REQUEST,
            "Enter a Lidarr API key to test the connection",
        ))?;
    let client = ArrClient::new(request.url.clone(), key).map_err(|_| {
        failure(
            StatusCode::BAD_REQUEST,
            "Enter a valid Lidarr HTTPS URL or node-loopback HTTP URL and API key",
        )
    })?;
    let status = fetch_status(&client).await.map_err(|_| {
        failure(
            StatusCode::BAD_GATEWAY,
            "Could not connect to Lidarr; check the URL and API key",
        )
    })?;
    validate_status(&status)?;
    Ok(Json(serde_json::json!({ "ok": true, "version": status.version })))
}

pub(super) async fn save(
    State(state): State<Arc<AppState>>,
    Json(mut request): Json<RadarrSettingsRequest>,
) -> ApiResult<ArrSettingsInfo> {
    let vault = vault(&state).await?;
    let key = request
        .api_key
        .take()
        .filter(|key| !key.trim().is_empty())
        .or(connection(&vault)?.map(|connection| connection.api_key.clone()))
        .ok_or(failure(
            StatusCode::BAD_REQUEST,
            "Enter a Lidarr API key for the first save",
        ))?;
    let client = ArrClient::new(request.url.clone(), key).map_err(|_| {
        failure(
            StatusCode::BAD_REQUEST,
            "Enter a valid Lidarr HTTPS URL or node-loopback HTTP URL and API key",
        )
    })?;
    let status = fetch_status(&client).await.map_err(|_| {
        failure(
            StatusCode::BAD_GATEWAY,
            "Could not connect to Lidarr; check the URL and API key",
        )
    })?;
    validate_status(&status)?;
    let key = client
        .api_key
        .to_str()
        .map_err(|_| failure(StatusCode::BAD_REQUEST, "Invalid API key"))?;
    vault
        .save_lidarr(Some(secure_config::ArrConnection {
            url: client.base_url.clone(),
            api_key: key.to_owned(),
        }))
        .map_err(|_| failure(StatusCode::INTERNAL_SERVER_ERROR, "Could not save encrypted Lidarr settings"))?;
    Ok(Json(ArrSettingsInfo {
        configured: true,
        persisted: true,
        url: Some(client.base_url),
        has_ui_auth: false,
    }))
}

pub(super) async fn clear(
    State(state): State<Arc<AppState>>,
) -> ApiResult<ArrSettingsInfo> {
    vault(&state)
        .await?
        .save_lidarr(None)
        .map_err(|_| failure(StatusCode::INTERNAL_SERVER_ERROR, "Could not clear Lidarr settings"))?;
    Ok(Json(ArrSettingsInfo {
        configured: false,
        persisted: false,
        url: None,
        has_ui_auth: false,
    }))
}

async fn fetch_list(
    client: &ArrClient,
    endpoint: &str,
) -> Result<Vec<serde_json::Value>, reqwest::Error> {
    client
        .http
        .get(format!("{}/api/v1/{endpoint}", client.base_url))
        .header("X-Api-Key", client.api_key.clone())
        .header(ACCEPT, "application/json")
        .send()
        .await?
        .error_for_status()?
        .json()
        .await
}

fn positive_id(value: &serde_json::Value, key: &str) -> Option<i64> {
    value.get(key).and_then(serde_json::Value::as_i64).filter(|id| *id > 0)
}

fn normalize_artist(
    artist: serde_json::Value,
    profiles: &[serde_json::Value],
) -> Option<serde_json::Value> {
    let id = positive_id(&artist, "id")?;
    let name = artist.get("artistName").and_then(serde_json::Value::as_str)
        .or_else(|| artist.get("name").and_then(serde_json::Value::as_str))?;
    let stats = &artist["statistics"];
    let profile = profiles.iter().find(|profile| profile["id"] == artist["qualityProfileId"]);
    Some(serde_json::json!({
        "provider": "lidarr",
        "id": id,
        "name": name,
        "foreignArtistId": artist.get("foreignArtistId"),
        "monitored": artist.get("monitored").and_then(serde_json::Value::as_bool).unwrap_or(false),
        "albumCount": stats.get("albumCount").and_then(serde_json::Value::as_u64).unwrap_or(0),
        "trackCount": stats.get("trackCount").and_then(serde_json::Value::as_u64).unwrap_or(0),
        "trackFileCount": stats.get("trackFileCount").and_then(serde_json::Value::as_u64).unwrap_or(0),
        "sizeOnDisk": stats.get("sizeOnDisk").and_then(serde_json::Value::as_u64).unwrap_or(0),
        "qualityProfile": profile.and_then(|profile| profile.get("name")).and_then(serde_json::Value::as_str),
        "path": artist.get("path"),
        "added": artist.get("added")
    }))
}

fn normalize_release(
    release: serde_json::Value,
    artist_names: &std::collections::HashMap<i64, String>,
) -> Option<serde_json::Value> {
    let id = positive_id(&release, "id")?;
    let artist_id = positive_id(&release, "artistId")?;
    let title = release.get("title").and_then(serde_json::Value::as_str)?;
    Some(serde_json::json!({
        "provider": "lidarr",
        "id": id,
        "artistId": artist_id,
        "artistName": artist_names.get(&artist_id),
        "title": title,
        "foreignAlbumId": release.get("foreignAlbumId"),
        "releaseDate": release.get("releaseDate"),
        "monitored": release.get("monitored").and_then(serde_json::Value::as_bool).unwrap_or(false),
        "trackCount": release["statistics"].get("trackCount").and_then(serde_json::Value::as_u64).unwrap_or(0),
        "trackFileCount": release["statistics"].get("trackFileCount").and_then(serde_json::Value::as_u64).unwrap_or(0),
        "sizeOnDisk": release["statistics"].get("sizeOnDisk").and_then(serde_json::Value::as_u64).unwrap_or(0),
        "path": release.get("path")
    }))
}

pub(super) async fn library(
    State(state): State<Arc<AppState>>,
) -> ApiResult<serde_json::Value> {
    let client = configured_client(&state).await?;
    let status = fetch_status(&client)
        .await
        .map_err(|_| failure(StatusCode::BAD_GATEWAY, "Lidarr status request failed"))?;
    validate_status(&status)?;
    let (artists, releases, profiles) = tokio::try_join!(
        fetch_list(&client, "artist"),
        fetch_list(&client, "album"),
        fetch_list(&client, "qualityprofile"),
    )
    .map_err(|_| failure(StatusCode::BAD_GATEWAY, "Lidarr library request failed"))?;

    let artists: Vec<_> = artists
        .into_iter()
        .filter_map(|artist| normalize_artist(artist, &profiles))
        .collect();
    let artist_names: std::collections::HashMap<_, _> = artists
        .iter()
        .filter_map(|artist| Some((artist["id"].as_i64()?, artist["name"].as_str()?.to_owned())))
        .collect();
    let releases: Vec<_> = releases
        .into_iter()
        .filter_map(|release| normalize_release(release, &artist_names))
        .collect();
    Ok(Json(serde_json::json!({
        "source": "lidarr",
        "version": status.version,
        "artists": artists,
        "releases": releases
    })))
}

#[cfg(test)]
#[path = "../../scripts/tests/test_rawrarr_lidarr.rs"]
mod tests;
