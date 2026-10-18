//! Read-only Sonarr adapter using the shared Arr transport and subscriber vault.
use super::*;
type ApiResult<T> = Result<Json<T>, (StatusCode, Json<ApiError>)>;
fn failure(status: StatusCode, error: &'static str) -> (StatusCode, Json<ApiError>) {
    (status, Json(ApiError { error }))
}
async fn vault(state: &AppState) -> Result<secure_config::AgeVault, (StatusCode, Json<ApiError>)> {
    state.vault.read().await.clone().ok_or(failure(StatusCode::LOCKED, "Unlock the age-encrypted configuration first"))
}
fn connection(vault: &secure_config::AgeVault) -> Result<Option<secure_config::ArrConnection>, (StatusCode, Json<ApiError>)> {
    Ok(vault.load().map_err(|_| failure(StatusCode::INTERNAL_SERVER_ERROR, "Could not read encrypted settings"))?.and_then(|c| c.sonarr.clone()))
}
async fn client(state: &AppState) -> Result<ArrClient, (StatusCode, Json<ApiError>)> {
    let config = connection(&vault(state).await?)?.ok_or(failure(StatusCode::SERVICE_UNAVAILABLE, "Sonarr is not configured"))?;
    ArrClient::new(config.url.clone(), config.api_key.clone()).map_err(|_| failure(StatusCode::BAD_REQUEST, "Invalid Sonarr settings"))
}
pub(super) async fn settings(State(state): State<Arc<AppState>>) -> ApiResult<RadarrSettingsInfo> {
    let config = connection(&vault(&state).await?)?;
    Ok(Json(RadarrSettingsInfo { configured: config.is_some(), persisted: config.is_some(), url: config.map(|c| c.url.clone()) }))
}
pub(super) async fn save(State(state): State<Arc<AppState>>, Json(mut request): Json<RadarrSettingsRequest>) -> ApiResult<RadarrSettingsInfo> {
    let vault = vault(&state).await?;
    let key = request.api_key.take().filter(|k| !k.trim().is_empty())
        .or(connection(&vault)?.map(|c| c.api_key.clone()))
        .ok_or(failure(StatusCode::BAD_REQUEST, "Enter a Sonarr API key for the first save"))?;
    let client = ArrClient::new(request.url.clone(), key).map_err(|_| failure(StatusCode::BAD_REQUEST, "Enter a valid Sonarr HTTPS URL and API key"))?;
    let status = client.status().await.map_err(|_| failure(StatusCode::BAD_GATEWAY, "Could not connect to Sonarr; check the URL and API key"))?;
    if !status.app_name.eq_ignore_ascii_case("sonarr") {
        return Err(failure(StatusCode::BAD_GATEWAY, "The service did not identify itself as Sonarr"));
    }
    let key = client.api_key.to_str().map_err(|_| failure(StatusCode::BAD_REQUEST, "Invalid API key"))?;
    vault.save_sonarr(Some(secure_config::ArrConnection { url: client.base_url.clone(), api_key: key.to_owned() }))
        .map_err(|_| failure(StatusCode::INTERNAL_SERVER_ERROR, "Could not save encrypted Sonarr settings"))?;
    Ok(Json(RadarrSettingsInfo { configured: true, persisted: true, url: Some(client.base_url) }))
}
pub(super) async fn clear(State(state): State<Arc<AppState>>) -> ApiResult<RadarrSettingsInfo> {
    vault(&state).await?.save_sonarr(None).map_err(|_| failure(StatusCode::INTERNAL_SERVER_ERROR, "Could not clear Sonarr settings"))?;
    Ok(Json(RadarrSettingsInfo { configured: false, persisted: false, url: None }))
}
pub(super) async fn status(State(state): State<Arc<AppState>>) -> ApiResult<RadarrStatus> {
    client(&state).await?.status().await.map(Json).map_err(|_| failure(StatusCode::BAD_GATEWAY, "Sonarr status request failed"))
}
fn normalize(series: serde_json::Value, profiles: &[serde_json::Value]) -> serde_json::Value {
    let stats = &series["statistics"];
    let profile = profiles.iter().find(|p| p["id"].as_i64().is_some() && p["id"] == series["qualityProfileId"]);
    serde_json::json!({
        "id": series["id"], "title": series["title"], "titleSlug": series["titleSlug"],
        "year": series["year"], "tvdbId": series["tvdbId"], "monitored": series["monitored"],
        "hasFile": stats["episodeFileCount"].as_u64().unwrap_or(0) > 0,
        "episodeFileCount": stats["episodeFileCount"].as_u64().unwrap_or(0),
        "episodeCount": stats["episodeCount"].as_u64().unwrap_or(0),
        "sizeOnDisk": stats["sizeOnDisk"].as_u64().unwrap_or(0),
        "qualityProfile": profile.and_then(|p| p["name"].as_str()),
        "path": series["path"], "added": series["added"]
    })
}
pub(super) async fn series(State(state): State<Arc<AppState>>) -> ApiResult<Vec<serde_json::Value>> {
    let client = client(&state).await?;
    async fn get(client: &ArrClient, resource: &str) -> Result<Vec<serde_json::Value>, reqwest::Error> {
        client.http.get(format!("{}/api/v3/{resource}", client.base_url))
            .header("X-Api-Key", client.api_key.clone()).send().await?.error_for_status()?.json().await
    }
    let (series, profiles) = tokio::try_join!(get(&client, "series"), get(&client, "qualityprofile"))
        .map_err(|_| failure(StatusCode::BAD_GATEWAY, "Sonarr library request failed"))?;
    Ok(Json(series.into_iter().map(|s| normalize(s, &profiles)).collect()))
}
#[cfg(test)]
#[path = "../../scripts/tests/test_rawrarr_sonarr.rs"]
mod tests;
