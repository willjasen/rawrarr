//! Read-only Sonarr adapter using the shared Arr transport and subscriber vault.
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
        .map_err(|_| {
            failure(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Could not read encrypted settings",
            )
        })?
        .and_then(|c| c.sonarr.clone()))
}
pub(super) async fn client(state: &AppState) -> Result<ArrClient, (StatusCode, Json<ApiError>)> {
    let config = connection(&vault(state).await?)?.ok_or(failure(
        StatusCode::SERVICE_UNAVAILABLE,
        "Sonarr is not configured",
    ))?;
    ArrClient::new(config.url.clone(), config.api_key.clone())
        .map_err(|_| failure(StatusCode::BAD_REQUEST, "Invalid Sonarr settings"))
}
pub(super) async fn settings(State(state): State<Arc<AppState>>) -> ApiResult<ArrSettingsInfo> {
    let config = connection(&vault(&state).await?)?;
    Ok(Json(ArrSettingsInfo {
        configured: config.is_some(),
        persisted: config.is_some(),
        url: config.map(|c| c.url.clone()),
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
            "Enter a Sonarr API key to test the connection",
        ))?;
    let client = ArrClient::new(request.url.clone(), key).map_err(|_| {
        failure(
            StatusCode::BAD_REQUEST,
            "Enter a valid Sonarr HTTPS URL and API key",
        )
    })?;
    let status = client.status().await.map_err(|_| {
        failure(
            StatusCode::BAD_GATEWAY,
            "Could not connect to Sonarr; check the URL and API key",
        )
    })?;
    if !status.app_name.eq_ignore_ascii_case("sonarr") {
        return Err(failure(
            StatusCode::BAD_GATEWAY,
            "The service did not identify itself as Sonarr",
        ));
    }
    Ok(Json(
        serde_json::json!({ "ok": true, "version": status.version }),
    ))
}
pub(super) async fn save(
    State(state): State<Arc<AppState>>,
    Json(mut request): Json<RadarrSettingsRequest>,
) -> ApiResult<ArrSettingsInfo> {
    let vault = vault(&state).await?;
    let key = request
        .api_key
        .take()
        .filter(|k| !k.trim().is_empty())
        .or(connection(&vault)?.map(|c| c.api_key.clone()))
        .ok_or(failure(
            StatusCode::BAD_REQUEST,
            "Enter a Sonarr API key for the first save",
        ))?;
    let client = ArrClient::new(request.url.clone(), key).map_err(|_| {
        failure(
            StatusCode::BAD_REQUEST,
            "Enter a valid Sonarr HTTPS URL and API key",
        )
    })?;
    let status = client.status().await.map_err(|_| {
        failure(
            StatusCode::BAD_GATEWAY,
            "Could not connect to Sonarr; check the URL and API key",
        )
    })?;
    if !status.app_name.eq_ignore_ascii_case("sonarr") {
        return Err(failure(
            StatusCode::BAD_GATEWAY,
            "The service did not identify itself as Sonarr",
        ));
    }
    let key = client
        .api_key
        .to_str()
        .map_err(|_| failure(StatusCode::BAD_REQUEST, "Invalid API key"))?;
    vault
        .save_sonarr(Some(secure_config::ArrConnection {
            url: client.base_url.clone(),
            api_key: key.to_owned(),
        }))
        .map_err(|_| {
            failure(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Could not save encrypted Sonarr settings",
            )
        })?;
    state.sonarr_scrape_notify.notify_one();
    Ok(Json(ArrSettingsInfo {
        configured: true,
        persisted: true,
        url: Some(client.base_url),
        has_ui_auth: false,
    }))
}
pub(super) async fn clear(State(state): State<Arc<AppState>>) -> ApiResult<ArrSettingsInfo> {
    vault(&state).await?.save_sonarr(None).map_err(|_| {
        failure(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Could not clear Sonarr settings",
        )
    })?;
    *state.sonarr_backfill.write().await = ArtworkBackfillProgress::default();
    state.sonarr_scrape_notify.notify_one();
    Ok(Json(ArrSettingsInfo {
        configured: false,
        persisted: false,
        url: None,
        has_ui_auth: false,
    }))
}
pub(super) async fn status(State(state): State<Arc<AppState>>) -> ApiResult<RadarrStatus> {
    client(&state)
        .await?
        .status()
        .await
        .map(Json)
        .map_err(|_| failure(StatusCode::BAD_GATEWAY, "Sonarr status request failed"))
}
fn normalize(series: serde_json::Value, profiles: &[serde_json::Value]) -> serde_json::Value {
    let stats = &series["statistics"];
    let profile = profiles
        .iter()
        .find(|p| p["id"].as_i64().is_some() && p["id"] == series["qualityProfileId"]);
    serde_json::json!({
        "id": series["id"], "title": series["title"], "titleSlug": series["titleSlug"],
        "year": series["year"], "tvdbId": series["tvdbId"], "monitored": series["monitored"],
        "hasFile": stats["episodeFileCount"].as_u64().unwrap_or(0) > 0,
        "episodeFileCount": stats["episodeFileCount"].as_u64().unwrap_or(0),
        "episodeCount": stats["episodeCount"].as_u64().unwrap_or(0),
        "sizeOnDisk": stats["sizeOnDisk"].as_u64().unwrap_or(0),
        "qualityProfile": profile.and_then(|p| p["name"].as_str()),
        "genres": series["genres"],
        "path": series["path"], "added": series["added"]
    })
}

pub(super) fn has_poster(series: &serde_json::Value) -> bool {
    poster_image(series).is_some_and(|image| image.get("url").and_then(|value| value.as_str()).is_some())
}

fn existing_artwork_is_valid(
    artwork: Result<Option<secure_config::EncryptedArtwork>, secure_config::VaultError>,
) -> Result<bool, ()> {
    match artwork {
        Ok(Some(_)) => Ok(true),
        Ok(None) | Err(secure_config::VaultError::DecryptionFailed) => Ok(false),
        Err(_) => Err(()),
    }
}

fn poster_image(series: &serde_json::Value) -> Option<&serde_json::Value> {
    series.get("images")?.as_array()?.iter().find(|image| {
        image.get("coverType").and_then(|value| value.as_str())
            .is_some_and(|kind| kind.eq_ignore_ascii_case("poster"))
    })
}

fn validated_tvdb_image_url(value: &str) -> Result<reqwest::Url, ()> {
    let url = reqwest::Url::parse(value).map_err(|_| ())?;
    if url.scheme() != "https"
        || url.host_str() != Some("artworks.thetvdb.com")
        || url.port().is_some_and(|port| port != 443)
        || !url.username().is_empty()
        || url.password().is_some()
        || !url.path().starts_with("/banners/")
    {
        return Err(());
    }
    Ok(url)
}

pub(super) async fn raw_series(client: &ArrClient) -> Result<Vec<serde_json::Value>, reqwest::Error> {
    client.http
        .get(format!("{}/api/v3/series", client.base_url))
        .header("X-Api-Key", client.api_key.clone())
        .header(ACCEPT, "application/json")
        .send().await?
        .error_for_status()?
        .json().await
}

pub(super) async fn raw_quality_profiles(client: &ArrClient) -> Result<Vec<serde_json::Value>, reqwest::Error> {
    client.http
        .get(format!("{}/api/v3/qualityprofile", client.base_url))
        .header("X-Api-Key", client.api_key.clone())
        .header(ACCEPT, "application/json")
        .send().await?
        .error_for_status()?
        .json().await
}

pub(super) async fn raw_episodes(
    client: &ArrClient,
    series_id: i64,
) -> Result<Vec<serde_json::Value>, reqwest::Error> {
    client.http
        .get(format!("{}/api/v3/episode", client.base_url))
        .header("X-Api-Key", client.api_key.clone())
        .header(ACCEPT, "application/json")
        .query(&[("seriesId", series_id)])
        .send().await?
        .error_for_status()?
        .json().await
}

fn attach_episodes(
    metadata: &mut serde_json::Value,
    fetched: Option<Vec<serde_json::Value>>,
    previous: Option<&serde_json::Value>,
) {
    let episodes = fetched.unwrap_or_else(|| {
        previous
            .and_then(|record| record.get("episodes"))
            .and_then(serde_json::Value::as_array)
            .cloned()
            .unwrap_or_default()
    });
    metadata["episodes"] = serde_json::Value::Array(episodes.into_iter().map(|mut episode| {
        metadata::sanitize_arr_text_fields(&mut episode);
        episode
    }).collect());
}

pub(super) async fn scrape_and_store(
    client: &ArrClient,
    vault: secure_config::AgeVault,
    series: &serde_json::Value,
    tvdb_id: u64,
) -> Result<bool, &'static str> {
    let series_id = series.get("id").and_then(serde_json::Value::as_i64).filter(|id| *id > 0).ok_or("invalid Sonarr series")?;
    let episodes = raw_episodes(client, series_id).await.ok();
    let mut metadata = metadata::arr_text_record("sonarr", "tvdbId", tvdb_id, series);
    let compare_vault = vault.clone();
    let current = tokio::task::spawn_blocking(move || compare_vault.load_tv_metadata(tvdb_id))
        .await.map_err(|_| "TV metadata load task failed")?.map_err(|_| "TV metadata load failed")?;
    attach_episodes(&mut metadata, episodes, current.as_ref());
    let serialized = serde_json::to_vec(&metadata).map_err(|_| "TV metadata serialization failed")?;
    if current.as_ref() != Some(&metadata) {
        let store_vault = vault.clone();
        tokio::task::spawn_blocking(move || store_vault.store_tv_metadata(tvdb_id, &serialized))
            .await.map_err(|_| "TV metadata store task failed")?.map_err(|_| "TV metadata store failed")?;
    }
    let artwork_vault = vault.clone();
    let existing_artwork = tokio::task::spawn_blocking(move || artwork_vault.load_tv_artwork(tvdb_id))
        .await.map_err(|_| "TV artwork load task failed")?;
    if existing_artwork_is_valid(existing_artwork).map_err(|_| "TV artwork cache check failed")? { return Ok(true); }
    let Some(image) = poster_image(series) else { return Ok(false); };
    if let Some(remote) = image.get("remoteUrl").and_then(serde_json::Value::as_str) {
        if let Ok(url) = validated_tvdb_image_url(remote) {
            if let Ok(mut response) = client.http.get(url).send().await {
                if let Ok(Some((content_type, bytes))) = super::read_image_response(&mut response).await {
                    let store_vault = vault.clone();
                    tokio::task::spawn_blocking(move || store_vault.store_tv_artwork(tvdb_id, &content_type, &bytes))
                        .await.map_err(|_| "TV artwork store task failed")?.map_err(|_| "TV artwork store failed")?;
                    return Ok(true);
                }
            }
        }
    }
    let Some(path) = image.get("url").and_then(|value| value.as_str()) else { return Ok(false); };
    if !path.starts_with('/') || path.starts_with("//") || path.contains('\\') { return Ok(false); }
    let base = reqwest::Url::parse(&client.base_url).map_err(|_| "invalid Sonarr base URL")?;
    let url = base.join(path).map_err(|_| "invalid Sonarr poster URL")?;
    if url.scheme() != base.scheme()
        || url.host_str() != base.host_str()
        || url.port_or_known_default() != base.port_or_known_default()
        || !url.username().is_empty() || url.password().is_some() {
        return Ok(false);
    }
    let mut response = client.http.get(url)
        .header("X-Api-Key", client.api_key.clone())
        .send().await.map_err(|_| "Sonarr poster request failed")?;
    let Some((content_type, bytes)) = super::read_image_response(&mut response).await.map_err(|_| "Sonarr poster response invalid")? else {
        return Ok(false);
    };
    let store_vault = vault.clone();
    tokio::task::spawn_blocking(move || store_vault.store_tv_artwork(tvdb_id, &content_type, &bytes))
        .await.map_err(|_| "TV artwork store task failed")?.map_err(|_| "TV artwork store failed")?;
    Ok(true)
}

pub(super) async fn series(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Vec<serde_json::Value>> {
    let client = client(&state).await?;
    let (series, profiles) = tokio::try_join!(raw_series(&client), async {
        client.http.get(format!("{}/api/v3/qualityprofile", client.base_url))
            .header("X-Api-Key", client.api_key.clone())
            .send().await?.error_for_status()?.json::<Vec<serde_json::Value>>().await
    })
            .map_err(|_| failure(StatusCode::BAD_GATEWAY, "Sonarr library request failed"))?;
    Ok(Json(
        series
            .into_iter()
            .map(|s| normalize(s, &profiles))
            .collect(),
    ))
}
#[cfg(test)]
#[path = "../../scripts/tests/test_rawrarr_sonarr.rs"]
mod tests;
