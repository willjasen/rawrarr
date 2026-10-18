#![forbid(unsafe_code)]

pub mod auth;
pub mod devices;
pub mod download_clients;
pub mod library;
pub mod metadata;
pub mod secure_config;
pub mod syncthing;

use axum::{
    Json, Router,
    body::Body,
    extract::{FromRef, Path as AxumPath, State},
    http::{StatusCode, header},
    middleware,
    response::{
        Html, IntoResponse, Redirect, Response, Sse,
        sse::{Event, KeepAlive},
    },
    routing::{get, post},
};
use futures_util::Stream;
use reqwest::header::{ACCEPT, HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use std::{env, net::SocketAddr, sync::Arc, time::Duration};
use tokio::sync::RwLock;
use zeroize::{Zeroize, Zeroizing};

#[derive(Clone)]
struct AppState {
    auth: auth::AuthState,
    radarr: Arc<RwLock<Option<RadarrClient>>>,
    vault: Arc<RwLock<Option<secure_config::AgeVault>>>,
    library: Option<library::LibraryRoot>,
    syncthing: Option<syncthing::Client>,
}

impl FromRef<AppState> for auth::AuthState {
    fn from_ref(state: &AppState) -> Self {
        state.auth.clone()
    }
}

impl FromRef<Arc<AppState>> for auth::AuthState {
    fn from_ref(state: &Arc<AppState>) -> Self {
        state.auth.clone()
    }
}

#[derive(Clone)]
struct RadarrClient {
    base_url: String,
    api_key: HeaderValue,
    http: reqwest::Client,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct RadarrStatus {
    app_name: String,
    version: String,
    url_base: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RadarrMovie {
    id: i64,
    title: String,
    year: Option<i64>,
    monitored: bool,
    has_file: bool,
    size_on_disk: u64,
    path: String,
    quality_profile: Option<String>,
    added: Option<String>,
    tmdb_id: Option<i64>,
}

#[derive(Serialize)]
struct ApiError {
    error: &'static str,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RadarrSettingsRequest {
    url: String,
    api_key: Option<String>,
}

impl Drop for RadarrSettingsRequest {
    fn drop(&mut self) {
        if let Some(api_key) = &mut self.api_key {
            api_key.zeroize();
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RadarrSettingsInfo {
    configured: bool,
    url: Option<String>,
    persisted: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AgeSettingsInfo {
    unlocked: bool,
    encrypted_config_exists: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ConfigurationView {
    configuration: Option<SharedConfiguration>,
    encryption: EncryptionVerification,
    sharing: SharingStatus,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SharedConfiguration {
    schema_version: u32,
    source: &'static str,
    radarr: Option<SharedRadarrConfig>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SharedRadarrConfig {
    url: String,
    api_key: &'static str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct EncryptionVerification {
    status: &'static str,
    ciphertext_exists: bool,
    age_format_recognized: bool,
    identity_unlocked: bool,
    decryption_verified: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SharingStatus {
    enabled: bool,
    status: &'static str,
}

#[derive(Deserialize)]
struct AgeIdentityRequest {
    identity: String,
}

impl Drop for AgeIdentityRequest {
    fn drop(&mut self) {
        self.identity.zeroize();
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GeneratedAgeIdentity {
    identity: String,
    recipient: String,
}

impl Drop for GeneratedAgeIdentity {
    fn drop(&mut self) {
        self.identity.zeroize();
    }
}

#[derive(Debug)]
enum RadarrConfigError {
    InvalidUrl,
    InvalidApiKey,
}

impl std::fmt::Display for RadarrConfigError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::InvalidUrl => "Invalid Radarr URL",
            Self::InvalidApiKey => "Invalid Radarr API key",
        })
    }
}

impl std::error::Error for RadarrConfigError {}

impl RadarrClient {
    fn from_env() -> Result<Option<Self>, Box<dyn std::error::Error>> {
        let Some(base_url) = env::var_os("RADARR_URL") else {
            return Ok(None);
        };
        let base_url = base_url
            .into_string()
            .map_err(|_| "RADARR_URL must be valid UTF-8")?;
        if base_url.trim().is_empty() {
            return Ok(None);
        }
        let api_key = env::var("RADARR_API_KEY")?;
        Ok(Some(Self::new(base_url, api_key)?))
    }

    fn new(base_url: String, api_key: String) -> Result<Self, RadarrConfigError> {
        let api_key = Zeroizing::new(api_key);
        if base_url.trim().is_empty() || base_url.len() > 2048 {
            return Err(RadarrConfigError::InvalidUrl);
        }
        let parsed =
            reqwest::Url::parse(base_url.trim()).map_err(|_| RadarrConfigError::InvalidUrl)?;
        if !matches!(parsed.scheme(), "http" | "https")
            || parsed.host_str().is_none()
            || !parsed.username().is_empty()
            || parsed.password().is_some()
            || parsed.query().is_some()
            || parsed.fragment().is_some()
        {
            return Err(RadarrConfigError::InvalidUrl);
        }
        if api_key.trim().is_empty() || api_key.len() > 4096 {
            return Err(RadarrConfigError::InvalidApiKey);
        }
        let mut api_key_header =
            HeaderValue::from_str(api_key.trim()).map_err(|_| RadarrConfigError::InvalidApiKey)?;
        api_key_header.set_sensitive(true);
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .map_err(|_| RadarrConfigError::InvalidUrl)?;

        Ok(Self {
            base_url: parsed.as_str().trim_end_matches('/').to_owned(),
            api_key: api_key_header,
            http,
        })
    }

    async fn status(&self) -> Result<RadarrStatus, reqwest::Error> {
        let mut headers = HeaderMap::new();
        headers.insert("X-Api-Key", self.api_key.clone());
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));

        self.http
            .get(format!("{}/api/v3/system/status", self.base_url))
            .headers(headers)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
    }

    async fn search_movies(
        &self,
        request: metadata::MovieSearchRequest,
    ) -> Result<metadata::MovieSearchResponse, metadata::SearchError> {
        let term = request.validate()?;
        let mut results: Vec<metadata::RadarrMovieLookupItem> = self
            .http
            .get(format!("{}/api/v3/movie/lookup", self.base_url))
            .header("X-Api-Key", self.api_key.clone())
            .header(ACCEPT, "application/json")
            .query(&[("term", term)])
            .send()
            .await
            .map_err(|_| metadata::SearchError::Upstream)?
            .error_for_status()
            .map_err(|_| metadata::SearchError::Upstream)?
            .json()
            .await
            .map_err(|_| metadata::SearchError::Upstream)?;
        results.truncate(5);
        Ok(metadata::MovieSearchResponse {
            provider: "radarr",
            candidates: results.into_iter().map(Into::into).collect(),
        })
    }

    async fn list_movies(&self) -> Result<Vec<RadarrMovie>, reqwest::Error> {
        let movies: Vec<serde_json::Value> = self
            .http
            .get(format!("{}/api/v3/movie", self.base_url))
            .header("X-Api-Key", self.api_key.clone())
            .header(ACCEPT, "application/json")
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        Ok(movies
            .into_iter()
            .map(|movie| {
                let profile = movie.get("qualityProfile");
                RadarrMovie {
                    id: movie.get("id").and_then(|v| v.as_i64()).unwrap_or_default(),
                    title: movie
                        .get("title")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Unknown title")
                        .to_owned(),
                    year: movie.get("year").and_then(|v| v.as_i64()),
                    monitored: movie
                        .get("monitored")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false),
                    has_file: movie
                        .get("hasFile")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false),
                    size_on_disk: movie
                        .get("sizeOnDisk")
                        .and_then(|v| v.as_u64())
                        .unwrap_or_default(),
                    path: movie
                        .get("path")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_owned(),
                    quality_profile: profile
                        .and_then(|v| v.get("name"))
                        .and_then(|v| v.as_str())
                        .map(str::to_owned),
                    added: movie
                        .get("added")
                        .and_then(|v| v.as_str())
                        .map(str::to_owned),
                    tmdb_id: movie.get("tmdbId").and_then(|v| v.as_i64()),
                }
            })
            .collect())
    }
}

async fn health() -> &'static str {
    "ok"
}

fn static_asset(content_type: &'static str, data: &'static [u8]) -> Response<Body> {
    let mut response = Response::new(Body::from(data));
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        axum::http::HeaderValue::from_static(content_type),
    );
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        axum::http::HeaderValue::from_static("public, max-age=86400"),
    );
    response
}

async fn rawrarr_logo() -> Response<Body> {
    static_asset("image/png", include_bytes!("assets/rawrarr-logo.png"))
}

async fn rawrarr_favicon() -> Response<Body> {
    static_asset(
        "image/svg+xml",
        include_str!("assets/rawrarr-favicon.svg").as_bytes(),
    )
}

async fn auth_page(State(state): State<Arc<AppState>>, headers: HeaderMap) -> Response<Body> {
    if auth::is_authenticated(&state.auth.pool, &headers).await {
        if state.vault.read().await.is_some() {
            return Redirect::to("/").into_response();
        }
        return Redirect::to("/setup/age").into_response();
    }
    auth::auth_html().into_response()
}

async fn dashboard(State(state): State<Arc<AppState>>, headers: HeaderMap) -> Response<Body> {
    if !auth::is_authenticated(&state.auth.pool, &headers).await {
        return Redirect::to("/auth").into_response();
    }
    if state.vault.read().await.is_none() {
        return Redirect::to("/setup/age").into_response();
    }
    Html(include_str!("ui.html")).into_response()
}

async fn age_setup_page(State(state): State<Arc<AppState>>) -> Response<Body> {
    if state.vault.read().await.is_some() {
        return Redirect::to("/").into_response();
    }
    auth::age_setup_html().into_response()
}

async fn logout(State(state): State<Arc<AppState>>, headers: HeaderMap) -> Response<Body> {
    *state.vault.write().await = None;
    *state.radarr.write().await = None;
    auth::end_session(&state.auth, &headers).await
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DashboardRadarr {
    configured: bool,
    connected: bool,
    app_name: Option<String>,
    version: Option<String>,
    url_base: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DashboardSnapshot {
    radarr: DashboardRadarr,
    metadata: MetadataStatus,
    library: library::LibraryInfo,
    devices: Vec<devices::DeviceInfo>,
}

async fn dashboard_snapshot(state: &AppState) -> DashboardSnapshot {
    let radarr_client = state.radarr.read().await.clone();
    let radarr = match radarr_client {
        Some(client) => match client.status().await {
            Ok(status) => DashboardRadarr {
                configured: true,
                connected: true,
                app_name: Some(status.app_name),
                version: Some(status.version),
                url_base: Some(status.url_base),
            },
            Err(_) => DashboardRadarr {
                configured: true,
                connected: false,
                app_name: None,
                version: None,
                url_base: None,
            },
        },
        None => DashboardRadarr {
            configured: false,
            connected: false,
            app_name: None,
            version: None,
            url_base: None,
        },
    };
    let library_name = state.library.as_ref().map(library::LibraryRoot::name);
    let library = match &state.library {
        Some(root) => library::LibraryInfo {
            configured: true,
            name: Some(root.name().to_owned()),
        },
        None => library::LibraryInfo {
            configured: false,
            name: None,
        },
    };

    DashboardSnapshot {
        metadata: MetadataStatus {
            configured: radarr.configured,
        },
        radarr,
        library,
        devices: vec![devices::DeviceInfo::local_scanner(library_name)],
    }
}

async fn dashboard_events(
    State(state): State<Arc<AppState>>,
) -> Sse<impl Stream<Item = Result<Event, std::convert::Infallible>>> {
    let stream = async_stream::stream! {
        let mut interval = tokio::time::interval(Duration::from_secs(30));
        loop {
            interval.tick().await;
            let snapshot = dashboard_snapshot(&state).await;
            if let Ok(event) = Event::default().event("dashboard").json_data(snapshot) {
                yield Ok(event);
            }
        }
    };
    Sse::new(stream).keep_alive(KeepAlive::default())
}

async fn library_info(State(state): State<Arc<AppState>>) -> Json<library::LibraryInfo> {
    match &state.library {
        Some(root) => Json(library::LibraryInfo {
            configured: true,
            name: Some(root.name().to_owned()),
        }),
        None => Json(library::LibraryInfo {
            configured: false,
            name: None,
        }),
    }
}

async fn list_devices(State(state): State<Arc<AppState>>) -> Json<Vec<devices::DeviceInfo>> {
    let library_name = state.library.as_ref().map(library::LibraryRoot::name);
    Json(vec![devices::DeviceInfo::local_scanner(library_name)])
}

async fn scan_library(
    State(state): State<Arc<AppState>>,
) -> Result<Json<library::ScanSummary>, (StatusCode, Json<ApiError>)> {
    let Some(root) = state.library.clone() else {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ApiError {
                error: "Local media folder is not configured",
            }),
        ));
    };

    tokio::task::spawn_blocking(move || root.scan())
        .await
        .map(Json)
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError {
                    error: "Local media scan failed",
                }),
            )
        })
}

async fn radarr_status(
    State(state): State<Arc<AppState>>,
) -> Result<Json<RadarrStatus>, (StatusCode, Json<ApiError>)> {
    let radarr = state.radarr.read().await.clone();
    let Some(radarr) = radarr else {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ApiError {
                error: "Radarr is not configured",
            }),
        ));
    };

    radarr.status().await.map(Json).map_err(|_| {
        (
            StatusCode::BAD_GATEWAY,
            Json(ApiError {
                error: "Radarr status request failed",
            }),
        )
    })
}

async fn radarr_movies(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<RadarrMovie>>, (StatusCode, Json<ApiError>)> {
    let radarr = state.radarr.read().await.clone();
    let Some(radarr) = radarr else {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ApiError {
                error: "Radarr is not configured",
            }),
        ));
    };
    radarr.list_movies().await.map(Json).map_err(|_| {
        (
            StatusCode::BAD_GATEWAY,
            Json(ApiError {
                error: "Radarr movie library request failed",
            }),
        )
    })
}

async fn search_movie_metadata(
    State(state): State<Arc<AppState>>,
    Json(request): Json<metadata::MovieSearchRequest>,
) -> Result<Json<metadata::MovieSearchResponse>, (StatusCode, Json<ApiError>)> {
    let provider = state.radarr.read().await.clone();
    let Some(provider) = provider else {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ApiError {
                error: "Movie metadata provider is not configured",
            }),
        ));
    };
    let mut result = provider.search_movies(request).await.map_err(|error| {
        let (status, message) = match error {
            metadata::SearchError::InvalidQuery | metadata::SearchError::InvalidYear => {
                (StatusCode::BAD_REQUEST, "Invalid movie search request")
            }
            metadata::SearchError::Upstream => {
                (StatusCode::BAD_GATEWAY, "Movie metadata search failed")
            }
        };
        (status, Json(ApiError { error: message }))
    })?;
    if let Some(vault) = state.vault.read().await.clone() {
        for candidate in &mut result.candidates {
            if let Some(source) = candidate.poster_source.take() {
                let Ok(source) = validated_tmdb_image_url(&source) else {
                    continue;
                };
                let id = candidate.tmdb_id;
                if !vault.artwork_exists(id) {
                    if let Ok(Some((content_type, bytes))) = fetch_tmdb_image(source).await {
                        let vault = vault.clone();
                        let _ = tokio::task::spawn_blocking(move || {
                            vault.store_artwork(id, &content_type, &bytes)
                        })
                        .await;
                    }
                }
                if vault.artwork_exists(id) {
                    candidate.poster_url = Some(format!("/api/metadata/movies/{id}/poster"));
                }
            }
        }
    }
    Ok(Json(result))
}

fn validated_tmdb_image_url(value: &str) -> Result<reqwest::Url, ()> {
    let url = reqwest::Url::parse(value).map_err(|_| ())?;
    if url.scheme() != "https"
        || url.host_str() != Some("image.tmdb.org")
        || url.port().is_some_and(|port| port != 443)
        || !url.username().is_empty()
        || url.password().is_some()
        || !url.path().starts_with("/t/p/")
    {
        return Err(());
    }
    Ok(url)
}

async fn fetch_tmdb_image(url: reqwest::Url) -> Result<Option<(String, Vec<u8>)>, ()> {
    const MAX_ARTWORK_BYTES: usize = 8 * 1024 * 1024;
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|_| ())?;
    let mut response = client.get(url).send().await.map_err(|_| ())?;
    if !response.status().is_success()
        || response
            .content_length()
            .is_some_and(|length| length > MAX_ARTWORK_BYTES as u64)
    {
        return Ok(None);
    }
    let content_type = response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(';').next())
        .map(str::trim)
        .filter(|value| matches!(*value, "image/jpeg" | "image/png" | "image/webp"))
        .ok_or(())?
        .to_owned();
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(|_| ())? {
        if bytes.len().saturating_add(chunk.len()) > MAX_ARTWORK_BYTES {
            return Ok(None);
        }
        bytes.extend_from_slice(&chunk);
    }
    if bytes.is_empty() {
        return Ok(None);
    }
    Ok(Some((content_type, bytes)))
}

async fn get_movie_poster(
    State(state): State<Arc<AppState>>,
    AxumPath(tmdb_id): AxumPath<u64>,
) -> Result<Response<Body>, (StatusCode, Json<ApiError>)> {
    let vault = state.vault.read().await.clone().ok_or((
        StatusCode::LOCKED,
        Json(ApiError {
            error: "Unlock the age-encrypted config to read cached artwork",
        }),
    ))?;
    let artwork = tokio::task::spawn_blocking(move || vault.load_artwork(tmdb_id))
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError {
                    error: "Could not decrypt cached artwork",
                }),
            )
        })?
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError {
                    error: "Could not decrypt cached artwork",
                }),
            )
        })?
        .ok_or((
            StatusCode::NOT_FOUND,
            Json(ApiError {
                error: "Encrypted artwork was not found",
            }),
        ))?;
    Response::builder()
        .header(header::CONTENT_TYPE, artwork.content_type)
        .header(header::CACHE_CONTROL, "private, no-store")
        .body(Body::from(artwork.bytes))
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError {
                    error: "Could not serve cached artwork",
                }),
            )
        })
}

#[derive(Serialize)]
struct MetadataStatus {
    configured: bool,
}

async fn metadata_status(State(state): State<Arc<AppState>>) -> Json<MetadataStatus> {
    Json(MetadataStatus {
        configured: state.radarr.read().await.is_some(),
    })
}

async fn get_radarr_settings(State(state): State<Arc<AppState>>) -> Json<RadarrSettingsInfo> {
    let radarr = state.radarr.read().await;
    Json(RadarrSettingsInfo {
        configured: radarr.is_some(),
        url: radarr.as_ref().map(|client| client.base_url.clone()),
        persisted: secure_config::AgeVault::config_path().is_file(),
    })
}

async fn get_age_settings(State(state): State<Arc<AppState>>) -> Json<AgeSettingsInfo> {
    Json(AgeSettingsInfo {
        unlocked: state.vault.read().await.is_some(),
        encrypted_config_exists: secure_config::AgeVault::config_path().is_file(),
    })
}

async fn get_configuration(State(state): State<Arc<AppState>>) -> Json<ConfigurationView> {
    let vault = state.vault.read().await.clone();
    let active_radarr = state.radarr.read().await.clone();
    let ciphertext_exists = secure_config::AgeVault::config_path().is_file();
    let age_format_recognized = secure_config::AgeVault::ciphertext_format_recognized();
    let mut decryption_verified = false;
    let mut stored_config = None;
    let mut verification_status = if ciphertext_exists {
        "locked"
    } else if active_radarr.is_some() {
        "not_encrypted"
    } else {
        "not_configured"
    };

    if ciphertext_exists && !age_format_recognized {
        verification_status = "unrecognized_ciphertext";
    } else if let Some(vault) = &vault {
        if ciphertext_exists {
            match vault.load() {
                Ok(Some(config)) => {
                    decryption_verified = true;
                    verification_status = "verified";
                    stored_config = Some(config);
                }
                Ok(None) => verification_status = "not_encrypted",
                Err(_) => verification_status = "verification_failed",
            }
        }
    }

    let source = if stored_config.is_some() {
        "age_encrypted_config"
    } else {
        "runtime_settings"
    };
    let configuration = stored_config
        .as_ref()
        .map(|config| (config.radarr_url.clone(), !config.radarr_api_key.is_empty()))
        .or_else(|| {
            active_radarr
                .as_ref()
                .map(|client| (client.base_url.clone(), true))
        })
        .map(|(url, has_api_key)| SharedConfiguration {
            schema_version: 1,
            source,
            radarr: Some(SharedRadarrConfig {
                url,
                api_key: if has_api_key {
                    "[REDACTED]"
                } else {
                    "[NOT SET]"
                },
            }),
        });

    Json(ConfigurationView {
        configuration,
        encryption: EncryptionVerification {
            status: verification_status,
            ciphertext_exists,
            age_format_recognized,
            identity_unlocked: vault.is_some(),
            decryption_verified,
        },
        sharing: SharingStatus {
            enabled: state.syncthing.is_some(),
            status: if state.syncthing.is_some() {
                "api_configured"
            } else {
                "not_configured"
            },
        },
    })
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SyncthingStatusResponse {
    configured: bool,
    reachable: bool,
    device_id: Option<String>,
    version: Option<String>,
}

async fn syncthing_status(State(state): State<Arc<AppState>>) -> Json<SyncthingStatusResponse> {
    let Some(client) = &state.syncthing else {
        return Json(SyncthingStatusResponse {
            configured: false,
            reachable: false,
            device_id: None,
            version: None,
        });
    };
    match client.status().await {
        Ok(status) => Json(SyncthingStatusResponse {
            configured: true,
            reachable: true,
            device_id: Some(status.my_id),
            version: Some(status.version),
        }),
        Err(_) => Json(SyncthingStatusResponse {
            configured: true,
            reachable: false,
            device_id: None,
            version: None,
        }),
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SyncthingShareResponse {
    status: &'static str,
    local_device_id: String,
    remote_device_id: String,
}

async fn configure_syncthing_share(
    State(state): State<Arc<AppState>>,
    Json(request): Json<syncthing::ShareRequest>,
) -> Result<Json<SyncthingShareResponse>, (StatusCode, Json<ApiError>)> {
    let Some(local) = &state.syncthing else {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ApiError {
                error: "Configure the local Syncthing API in the container first",
            }),
        ));
    };
    let remote =
        syncthing::Client::new(&request.remote_api_url, &request.remote_api_key).map_err(|_| {
            (
                StatusCode::BAD_REQUEST,
                Json(ApiError {
                    error: "Invalid remote Syncthing connection details",
                }),
            )
        })?;
    let (local_status, remote_status) = local.share_with(&remote, &request).await.map_err(|error| {
        let status = match error {
            syncthing::Error::InvalidConfiguration => StatusCode::BAD_REQUEST,
            syncthing::Error::Request | syncthing::Error::Rejected => StatusCode::BAD_GATEWAY,
        };
        (status, Json(ApiError { error: "Syncthing could not configure the peer share; check API access, paths, and addresses" }))
    })?;
    Ok(Json(SyncthingShareResponse {
        status: "configured",
        local_device_id: local_status.my_id,
        remote_device_id: remote_status.my_id,
    }))
}

async fn generate_age_identity() -> Result<Json<GeneratedAgeIdentity>, (StatusCode, Json<ApiError>)>
{
    let (identity, recipient) = secure_config::AgeVault::generate_identity().map_err(|_| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ApiError {
                error: "Post-quantum age tools are unavailable",
            }),
        )
    })?;
    Ok(Json(GeneratedAgeIdentity {
        identity,
        recipient,
    }))
}

async fn unlock_age_settings(
    State(state): State<Arc<AppState>>,
    Json(mut request): Json<AgeIdentityRequest>,
) -> Result<Json<AgeSettingsInfo>, (StatusCode, Json<ApiError>)> {
    let vault = secure_config::AgeVault::from_identity(std::mem::take(&mut request.identity))
        .map_err(|_| {
            (
                StatusCode::BAD_REQUEST,
                Json(ApiError {
                    error: "Import a valid post-quantum age identity",
                }),
            )
        })?;
    let stored = vault.load().map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                error: "Could not decrypt the saved config with this identity",
            }),
        )
    })?;
    if let Some(settings) = stored {
        let client =
            RadarrClient::new(settings.radarr_url.clone(), settings.radarr_api_key.clone())
                .map_err(|_| {
                    (
                        StatusCode::BAD_REQUEST,
                        Json(ApiError {
                            error: "Saved Radarr settings are invalid",
                        }),
                    )
                })?;
        *state.radarr.write().await = Some(client);
    }
    let exists = vault.config_exists();
    *state.vault.write().await = Some(vault);
    Ok(Json(AgeSettingsInfo {
        unlocked: true,
        encrypted_config_exists: exists,
    }))
}

async fn put_radarr_settings(
    State(state): State<Arc<AppState>>,
    Json(mut settings): Json<RadarrSettingsRequest>,
) -> Result<Json<RadarrSettingsInfo>, (StatusCode, Json<ApiError>)> {
    let vault = state.vault.read().await.clone().ok_or((
        StatusCode::LOCKED,
        Json(ApiError {
            error: "Unlock the age-encrypted config before saving Radarr settings",
        }),
    ))?;
    let supplied_api_key = settings
        .api_key
        .take()
        .filter(|api_key| !api_key.trim().is_empty());
    let api_key = match supplied_api_key {
        Some(api_key) => api_key,
        None => state
            .radarr
            .read()
            .await
            .as_ref()
            .and_then(|radarr| radarr.api_key.to_str().ok().map(str::to_owned))
            .ok_or((
                StatusCode::BAD_REQUEST,
                Json(ApiError {
                    error: "Enter a Radarr API key for the first save",
                }),
            ))?,
    };
    let client =
        RadarrClient::new(std::mem::take(&mut settings.url), api_key).map_err(|error| {
            let message = match error {
                RadarrConfigError::InvalidUrl => "Enter a valid Radarr HTTP or HTTPS URL",
                RadarrConfigError::InvalidApiKey => "Enter a valid Radarr API key",
            };
            (StatusCode::BAD_REQUEST, Json(ApiError { error: message }))
        })?;
    let url = client.base_url.clone();
    let api_key = client.api_key.to_str().map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                error: "Enter a valid Radarr API key",
            }),
        )
    })?;
    vault
        .save(&secure_config::PersistedConfig {
            radarr_url: url.clone(),
            radarr_api_key: api_key.to_owned(),
        })
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError {
                    error: "Could not save the encrypted config",
                }),
            )
        })?;
    *state.radarr.write().await = Some(client);
    Ok(Json(RadarrSettingsInfo {
        configured: true,
        url: Some(url),
        persisted: true,
    }))
}

async fn delete_radarr_settings(
    State(state): State<Arc<AppState>>,
) -> Result<Json<RadarrSettingsInfo>, (StatusCode, Json<ApiError>)> {
    if let Some(vault) = state.vault.read().await.clone() {
        vault.clear().map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError {
                    error: "Could not remove the encrypted config",
                }),
            )
        })?;
    } else if secure_config::AgeVault::config_path().is_file() {
        return Err((
            StatusCode::LOCKED,
            Json(ApiError {
                error: "Unlock the age-encrypted config before clearing Radarr settings",
            }),
        ));
    }
    *state.radarr.write().await = None;
    Ok(Json(RadarrSettingsInfo {
        configured: false,
        url: None,
        persisted: false,
    }))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address = env::var("RUST_ARR_LISTEN_ADDR").unwrap_or_else(|_| "127.0.0.1:8080".to_owned());
    let socket_address: SocketAddr = address.parse()?;
    let auth = auth::AuthState::connect().await?;
    let mut radarr = RadarrClient::from_env()?;
    let mut vault = None;
    if let Some(identity_path) =
        env::var_os("RAWARR_AGE_IDENTITY_FILE").filter(|path| !path.is_empty())
    {
        let identity = std::fs::read_to_string(identity_path)?;
        let unlocked = secure_config::AgeVault::from_identity(identity)?;
        if let Some(settings) = unlocked.load()? {
            radarr = Some(RadarrClient::new(
                settings.radarr_url.clone(),
                settings.radarr_api_key.clone(),
            )?);
        }
        vault = Some(unlocked);
    }
    let state = Arc::new(AppState {
        auth: auth.clone(),
        radarr: Arc::new(RwLock::new(radarr)),
        vault: Arc::new(RwLock::new(vault)),
        library: library::LibraryRoot::from_env()?,
        syncthing: syncthing::Client::from_env()
            .map_err(|_| "invalid Syncthing environment configuration")?,
    });

    let protected = Router::new()
        .route("/", get(dashboard))
        .route("/setup/age", get(age_setup_page))
        .route("/api/auth/logout", post(logout))
        .route("/api/events", get(dashboard_events))
        .route("/api/library", get(library_info))
        .route("/api/devices", get(list_devices))
        .route("/api/syncthing/status", get(syncthing_status))
        .route("/api/syncthing/share", post(configure_syncthing_share))
        .route("/api/library/scan", axum::routing::post(scan_library))
        .route(
            "/api/metadata/movies/search",
            axum::routing::post(search_movie_metadata),
        )
        .route(
            "/api/metadata/movies/{tmdb_id}/poster",
            get(get_movie_poster),
        )
        .route("/api/metadata/status", get(metadata_status))
        .route(
            "/api/settings/radarr",
            get(get_radarr_settings)
                .put(put_radarr_settings)
                .delete(delete_radarr_settings),
        )
        .route(
            "/api/settings/age",
            get(get_age_settings).post(unlock_age_settings),
        )
        .route(
            "/api/settings/age/generate",
            axum::routing::post(generate_age_identity),
        )
        .route("/api/configuration", get(get_configuration))
        .route("/api/radarr/status", get(radarr_status))
        .route("/api/radarr/movies", get(radarr_movies))
        .route_layer(middleware::from_fn_with_state(
            auth.pool.clone(),
            auth::require_session,
        ));
    let app = Router::new()
        .route("/health", get(health))
        .route("/assets/rawrarr-logo.png", get(rawrarr_logo))
        .route("/assets/rawrarr-favicon.svg", get(rawrarr_favicon))
        .route("/auth", get(auth_page))
        .route("/api/auth/status", get(auth::status))
        .route("/api/auth/register/start", post(auth::begin_registration))
        .route("/api/auth/register/finish", post(auth::finish_registration))
        .route("/api/auth/login/start", post(auth::begin_authentication))
        .route("/api/auth/login/finish", post(auth::finish_authentication))
        .merge(protected)
        .with_state(state);
    let listener = tokio::net::TcpListener::bind(socket_address).await?;

    println!("rawrarr listening on {socket_address}");
    axum::serve(listener, app).await?;
    Ok(())
}

#[cfg(test)]
mod metadata_image_tests {
    use super::validated_tmdb_image_url;

    #[test]
    fn metadata_image_urls_are_restricted_to_tmdb_image_paths() {
        assert!(validated_tmdb_image_url("https://image.tmdb.org/t/p/original/poster.jpg").is_ok());
        assert!(validated_tmdb_image_url("http://image.tmdb.org/t/p/original/poster.jpg").is_err());
        assert!(
            validated_tmdb_image_url(
                "https://image.tmdb.org.attacker.invalid/t/p/original/poster.jpg"
            )
            .is_err()
        );
        assert!(
            validated_tmdb_image_url("https://user@image.tmdb.org/t/p/original/poster.jpg")
                .is_err()
        );
    }
}
