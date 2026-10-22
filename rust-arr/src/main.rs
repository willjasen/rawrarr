#![forbid(unsafe_code)]

mod movie_snapshot;
mod sonarr;

pub mod auth;
mod coordinator;
mod device_signing;
mod artwork_cache;
mod coordinator_avatar;
pub mod devices;
pub mod download_clients;
pub mod library;
pub mod metadata;
mod node_auth;
mod node_role;
mod quality_profiles;
pub mod secure_config;
pub mod shared_logs;
pub mod storage_node;
pub mod syncthing;
mod ui_assets;

use axum::middleware::Next;
use axum::{
    Json, Router,
    body::{Body, Bytes},
    extract::{Path as AxumPath, Request, State},
    http::{StatusCode, header},
    middleware,
    response::{
        Html, IntoResponse, Redirect, Response, Sse,
        sse::{Event, KeepAlive},
    },
    routing::{get, post},
};
use aes_gcm::KeyInit;
use p256::elliptic_curve::{rand_core::RngCore, sec1::ToEncodedPoint};
use futures_util::Stream;
use reqwest::header::{ACCEPT, HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    env,
    net::SocketAddr,
    sync::atomic::{AtomicBool, Ordering},
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::RwLock;
use zeroize::{Zeroize, Zeroizing};

#[derive(Clone)]
struct AppState {
    auth: node_auth::NodeAuth,
    role: node_role::NodeRole,
    radarr: Arc<RwLock<Option<RadarrClient>>>,
    vault: Arc<RwLock<Option<secure_config::AgeVault>>>,
    library: Option<library::LibraryRoot>,
    device_id: String,
    shared_logs: Option<shared_logs::SharedLogger>,
    storage_scan: storage_node::ScanController,
    syncthing: Option<syncthing::Client>,
    artwork_backfill: Arc<RwLock<ArtworkBackfillProgress>>,
    sonarr_backfill: Arc<RwLock<ArtworkBackfillProgress>>,
    // Shared across both Arr workers so their metadata requests never overlap.
    artwork_scrape_lock: Arc<tokio::sync::Mutex<()>>,
    artwork_scrape_notify: Arc<tokio::sync::Notify>,
    artwork_force_rescan: Arc<AtomicBool>,
    sonarr_scrape_notify: Arc<tokio::sync::Notify>,
    artwork_cache: Arc<artwork_cache::ArtworkCache>,
}

#[derive(Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
struct ArtworkBackfillProgress {
    active: bool,
    completed: usize,
    scraped: usize,
    total: usize,
    failed: usize,
}

#[derive(Clone)]
struct ArrClient {
    movie_snapshot: movie_snapshot::Cache,
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

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct RadarrMovie {
    id: i64,
    title: String,
    title_slug: Option<String>,
    year: Option<i64>,
    monitored: bool,
    has_file: bool,
    size_on_disk: u64,
    path: String,
    quality_profile: Option<String>,
    added: Option<String>,
    tmdb_id: Option<i64>,
    poster_url: Option<String>,
    artwork_source: Option<String>,
    genres: Vec<String>,
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
    theme_seed: Option<String>,
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
    metadata_filename_key_configured: bool,
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
#[serde(rename_all = "camelCase")]
struct AgeIdentityRequest {
    identity: String,
    #[serde(default)]
    remember_on_device: bool,
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
    theme_seed: String,
}

fn quiet_logs_enabled(value: Option<&str>) -> bool {
    value.is_some_and(|value| {
        matches!(
            value.trim().to_ascii_lowercase().as_str(),
            "1" | "true" | "yes" | "on"
        )
    })
}

async fn log_request(request: Request<Body>, next: Next) -> Response<Body> {
    let method = request.method().clone();
    // Deliberately exclude query parameters, which can contain sensitive values.
    let path = request.uri().path().to_owned();
    let started = Instant::now();
    let response = next.run(request).await;
    let status = response.status();
    let elapsed_ms = started.elapsed().as_millis();
    if !quiet_logs_enabled(env::var("RAWARR_QUIET_LOGS").ok().as_deref()) {
        println!(
            "request {method} {path} -> {} ({elapsed_ms} ms)",
            status.as_u16()
        );
    } else if status.is_server_error() {
        eprintln!(
            "request {method} {path} -> {} ({elapsed_ms} ms)",
            status.as_u16()
        );
    }
    response
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

type RadarrClient = ArrClient;

impl ArrClient {
    fn new(base_url: String, api_key: String) -> Result<Self, RadarrConfigError> {
        let api_key = Zeroizing::new(api_key);
        if base_url.trim().is_empty() || base_url.len() > 2048 {
            return Err(RadarrConfigError::InvalidUrl);
        }
        let parsed =
            reqwest::Url::parse(base_url.trim()).map_err(|_| RadarrConfigError::InvalidUrl)?;
        if parsed.scheme() != "https"
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
            // Arr library endpoints can take several seconds while the service
            // is scraping metadata; five seconds caused intermittent 502s.
            .timeout(Duration::from_secs(30))
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|_| RadarrConfigError::InvalidUrl)?;

        Ok(Self {
            movie_snapshot: Default::default(),
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

    async fn list_movies(
        &self,
        vault: Option<secure_config::AgeVault>,
    ) -> Result<Vec<RadarrMovie>, &'static str> {
        let movies: Vec<serde_json::Value> = self
            .http
            .get(format!("{}/api/v3/movie", self.base_url))
            .header("X-Api-Key", self.api_key.clone())
            .header(ACCEPT, "application/json")
            .send()
            .await
            .map_err(|_| "Radarr movie library request failed")?
            .error_for_status()
            .map_err(|_| "Radarr movie library request failed")?
            .json()
            .await
            .map_err(|_| "Radarr movie library request failed")?;
        let movies = quality_profiles::enrich(self, vault.clone(), movies).await?;
        Ok(movies
            .into_iter()
            .map(|movie| {
                let profile = movie.get("qualityProfile");
                let tmdb_id = movie.get("tmdbId").and_then(|v| v.as_i64());
                let poster_url = movie
                    .get("images")
                    .and_then(serde_json::Value::as_array)
                    .and_then(|images| {
                        images.iter().find(|image| {
                            image
                                .get("coverType")
                                .and_then(|v| v.as_str())
                                .is_some_and(|kind| kind.eq_ignore_ascii_case("poster"))
                                && (image.get("remoteUrl").and_then(|v| v.as_str()).is_some()
                                    || image.get("url").and_then(|v| v.as_str()).is_some())
                        })
                    })
                    .and_then(|_| tmdb_id)
                    .map(|id| format!("/api/radarr/movies/{id}/poster"));
                let artwork_source = tmdb_id.filter(|_| poster_url.is_some()).map(|id| {
                    let cached = u64::try_from(id).ok().is_some_and(|id| vault.as_ref().is_some_and(|vault| vault.artwork_exists(id))) || secure_config::AgeVault::metadata_dir()
                        .join("movies").join("artwork").join(format!("{id}.aged")).is_file();
                    if cached { "shared" } else { "radarr" }.to_owned()
                });
                RadarrMovie {
                    id: movie.get("id").and_then(|v| v.as_i64()).unwrap_or_default(),
                    title: movie
                        .get("title")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Unknown title")
                        .to_owned(),
                    title_slug: movie
                        .get("titleSlug")
                        .and_then(|v| v.as_str())
                        .map(str::to_owned),
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
                    tmdb_id,
                    poster_url,
                    artwork_source,
                    genres: movie.get("genres").and_then(serde_json::Value::as_array).into_iter().flatten()
                        .filter_map(serde_json::Value::as_str).map(str::to_owned).collect(),
                }
            })
            .collect())
    }

    async fn movie_by_tmdb_id(
        &self,
        tmdb_id: u64,
    ) -> Result<Option<serde_json::Value>, reqwest::Error> {
        if let Some(movie) = self.movie_snapshot.poster_movie(tmdb_id).await {
            return Ok(Some(movie));
        }
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
            .find(|movie| movie.get("tmdbId").and_then(|id| id.as_u64()) == Some(tmdb_id)))
    }

    async fn fetch_movie_poster(
        &self,
        movie: &serde_json::Value,
    ) -> Result<Option<(String, Vec<u8>)>, ()> {
        let Some(image) = poster_image(movie) else {
            return Ok(None);
        };
        if let Some(remote) = image.get("remoteUrl").and_then(|value| value.as_str()) {
            if let Ok(url) = validated_tmdb_image_url(remote) {
                if let Ok(Some(result)) = fetch_tmdb_image(url).await {
                    return Ok(Some(result));
                }
            }
        }
        let Some(path) = image.get("url").and_then(|value| value.as_str()) else {
            return Ok(None);
        };
        if !path.starts_with('/') || path.starts_with("//") || path.contains('\\') {
            return Ok(None);
        }
        let base = reqwest::Url::parse(&self.base_url).map_err(|_| ())?;
        let url = base.join(path).map_err(|_| ())?;
        if url.scheme() != base.scheme()
            || url.host_str() != base.host_str()
            || url.port_or_known_default() != base.port_or_known_default()
            || !url.username().is_empty()
            || url.password().is_some()
        {
            return Ok(None);
        }
        let mut response = self
            .http
            .get(url)
            .header("X-Api-Key", self.api_key.clone())
            .send()
            .await
            .map_err(|_| ())?;
        read_image_response(&mut response).await
    }
}

fn poster_image(movie: &serde_json::Value) -> Option<&serde_json::Value> {
    movie.get("images")?.as_array()?.iter().find(|image| {
        image
            .get("coverType")
            .and_then(|value| value.as_str())
            .is_some_and(|kind| kind.eq_ignore_ascii_case("poster"))
    })
}

async fn health() -> Html<&'static str> {
    Html(
        r##"<!doctype html>
<html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><meta name="theme-color" content="#101514"><title>rawrarr — System health</title>
<style>
:root{color-scheme:dark;--bg:#101514;--panel:#171e1a;--line:#303b33;--text:#edf3ed;--muted:#9aa89e;--green:#b8f36a;font:16px Inter,ui-sans-serif,system-ui,-apple-system,"Segoe UI",sans-serif;background:var(--bg);color:var(--text)}*{box-sizing:border-box}body{margin:0;min-height:100vh;background:radial-gradient(ellipse at 75% 0,#263322 0,transparent 42%),var(--bg)}main{width:min(760px,100%);margin:auto;padding:64px 24px}.brand{color:var(--green);font-size:13px;letter-spacing:.14em;text-transform:uppercase;font-weight:750}.eyebrow{margin-top:62px;color:var(--muted);font-size:12px;text-transform:uppercase;letter-spacing:.14em}h1{font-size:clamp(36px,8vw,62px);letter-spacing:-.06em;margin:10px 0 12px}.summary{color:var(--muted);line-height:1.65;max-width:520px}.state{display:inline-flex;align-items:center;gap:9px;margin-top:22px;border:1px solid #435638;background:#202a1b;color:#d5f7aa;border-radius:999px;padding:9px 13px;font-size:13px}.dot{width:9px;height:9px;border-radius:50%;background:var(--green);box-shadow:0 0 14px #b8f36a99}.panel{margin-top:42px;border:1px solid var(--line);background:var(--panel);border-radius:16px;overflow:hidden}.panel h2{font-size:14px;margin:0;padding:19px 21px;border-bottom:1px solid var(--line)}.check{display:flex;justify-content:space-between;align-items:center;padding:17px 21px;border-bottom:1px solid #27312a}.check:last-child{border:0}.check strong{font-size:14px;font-weight:600}.check span{font-size:12px;color:var(--muted)}.ok{color:var(--green)!important}.foot{margin-top:24px;color:#718076;font-size:12px}a{color:var(--green);text-decoration:none}a:hover{text-decoration:underline}@media(max-width:520px){main{padding:36px 20px}.eyebrow{margin-top:48px}.check{gap:12px}.check span{text-align:right}}
</style></head><body><main><div class="brand">rawrarr</div><div class="eyebrow">System status</div><h1>All systems ready.</h1><p class="summary">The rawrarr service is responding and ready to handle requests. This page checks the web process; connected services are reported separately in the app.</p><div class="state"><i class="dot"></i>Operational</div><section class="panel" aria-label="Health checks"><h2>Service checks</h2><div class="check"><strong>Web service</strong><span class="ok">● &nbsp;Responding</span></div><div class="check"><strong>Health endpoint</strong><span>/health · HTTP 200</span></div><div class="check"><strong>Checked</strong><span id="checked">Just now</span></div></section><p class="foot"><a href="/">Open rawrarr</a> &nbsp;·&nbsp; Auto-refreshes every 30 seconds</p></main><script>const checked=document.getElementById('checked');function update(){checked.textContent=new Intl.DateTimeFormat(undefined,{hour:'numeric',minute:'2-digit',second:'2-digit'}).format(new Date())}update();setInterval(()=>location.reload(),30000)</script></body></html>"##,
    )
}

async fn direct_health(State(state): State<Arc<AppState>>) -> Response {
    let cache = state.artwork_cache.clone();
    let (configured, reachable) = tokio::task::spawn_blocking(move || {
        let configured = cache.is_configured();
        (configured, configured && cache.ping())
    }).await.unwrap_or((false, false));
    Json(serde_json::json!({
        "status": "ok",
        "artworkCacheConfigured": configured,
        "artworkCacheReachable": reachable
    })).into_response()
}

fn static_asset(content_type: &'static str, data: impl Into<Body>) -> Response<Body> {
    let mut response = Response::new(data.into());
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
    static_asset("image/png", ui_assets::bytes("assets/rawrarr-logo.png", include_bytes!("assets/rawrarr-logo.png")).await)
}

async fn rawrarr_logo_jxl() -> Response<Body> {
    static_asset("image/jxl", ui_assets::bytes("assets/rawrarr-logo.jxl", include_bytes!("assets/rawrarr-logo.jxl")).await)
}

async fn rawrarr_favicon() -> Response<Body> {
    static_asset(
        "image/svg+xml",
        ui_assets::text("assets/rawrarr-favicon.svg", include_str!("assets/rawrarr-favicon.svg")).await.into_bytes(),
    )
}

async fn passkey_client_js() -> Response<Body> {
    static_asset(
        "text/javascript; charset=utf-8",
        ui_assets::text("passkey-client.mjs", include_str!("passkey-client.mjs")).await.into_bytes(),
    )
}

async fn metadata_export_js() -> Response<Body> {
    let mut response = static_asset(
        "text/javascript; charset=utf-8",
        ui_assets::bytes("assets/metadata-export.js", include_bytes!("assets/metadata-export.js")).await,
    );
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        axum::http::HeaderValue::from_static("private, no-cache"),
    );
    response
}

async fn auth_page(State(state): State<Arc<AppState>>, headers: HeaderMap) -> Response<Body> {
    if state.auth.authenticated(&headers).await {
        if state.vault.read().await.is_some() {
            return Redirect::to("/").into_response();
        }
        return Redirect::to("/setup/age").into_response();
    }
    auth::auth_html().await.into_response()
}

async fn dashboard(State(state): State<Arc<AppState>>, headers: HeaderMap) -> Response<Body> {
    if !state.auth.authenticated(&headers).await {
        return Redirect::to("/auth").into_response();
    }
    if state.vault.read().await.is_none() {
        return Redirect::to("/setup/age").into_response();
    }
    Html(ui_assets::text("ui.html", include_str!("ui.html")).await).into_response()
}

async fn age_setup_page(State(state): State<Arc<AppState>>) -> Response<Body> {
    if state.vault.read().await.is_some() {
        return Redirect::to("/").into_response();
    }
    auth::age_setup_html().await.into_response()
}

async fn logout(State(state): State<Arc<AppState>>, headers: HeaderMap) -> Response<Body> {
    *state.vault.write().await = None;
    *state.radarr.write().await = None;
    state.auth.logout(&headers).await
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
        devices: vec![devices::DeviceInfo::local_scanner(
            library_name,
            &state.device_id,
        )],
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
    Json(vec![devices::DeviceInfo::local_scanner(
        library_name,
        &state.device_id,
    )])
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

    let vault = state.vault.read().await.clone().ok_or((
        StatusCode::LOCKED,
        Json(ApiError {
            error: "Unlock the age identity before scanning to create encrypted shared metadata",
        }),
    ))?;
    let device_id = state.device_id.clone();

    let summary = tokio::task::spawn_blocking(move || {
        let mut summary = root.scan();
        for file in &summary.items {
            let sidecar = library::MediaSidecar {
                schema_version: 1,
                device_id: device_id.clone(),
                library_name: summary.library_name.clone(),
                scanned_at_unix_seconds: summary.scanned_at_unix_seconds,
                file: file.clone(),
            };
            match serde_json::to_vec(&sidecar)
                .map_err(|_| secure_config::VaultError::InvalidConfig)
                .and_then(|json| {
                    vault.store_library_sidecar(
                        &device_id,
                        &summary.library_name,
                        &file.path,
                        &json,
                    )
                }) {
                Ok(()) => summary.sidecars_created += 1,
                Err(_) => summary.sidecar_errors += 1,
            }
        }
        summary
    })
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                error: "Local media scan failed",
            }),
        )
    })?;
    if let Some(logger) = &state.shared_logs {
        logger.scan_completed(
            summary.items.len(),
            summary.sidecars_created,
            summary.sidecar_errors,
            summary.issues,
            summary.truncated,
        );
    }
    println!(
        "library scan completed: {} files; {} encrypted sidecars; {} sidecar errors; {} issues; truncated={}",
        summary.items.len(),
        summary.sidecars_created,
        summary.sidecar_errors,
        summary.issues,
        summary.truncated
    );
    Ok(Json(summary))
}

async fn request_storage_scan(
    State(state): State<Arc<AppState>>,
    Json(options): Json<storage_node::ScanOptions>,
) -> Response {
    if state.role != node_role::NodeRole::Storage {
        return StatusCode::NOT_FOUND.into_response();
    }
    match state.storage_scan.start(options, state.shared_logs.clone()).await {
        Ok(()) => (StatusCode::ACCEPTED, Json(state.storage_scan.status().await)).into_response(),
        Err(error) => (StatusCode::CONFLICT, error).into_response(),
    }
}

async fn storage_scan_status(State(state): State<Arc<AppState>>) -> Response {
    if state.role != node_role::NodeRole::Storage {
        return StatusCode::NOT_FOUND.into_response();
    }
    Json(state.storage_scan.status().await).into_response()
}

async fn cancel_storage_scan(State(state): State<Arc<AppState>>) -> Response {
    if state.role != node_role::NodeRole::Storage {
        return StatusCode::NOT_FOUND.into_response();
    }
    match state.storage_scan.cancel().await {
        Ok(status) => Json(status).into_response(),
        Err(error) => (StatusCode::CONFLICT, error).into_response(),
    }
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
    let vault = state.vault.read().await.clone();
    let movies = radarr
        .list_movies(vault)
        .await
        .map_err(|error| (StatusCode::BAD_GATEWAY, Json(ApiError { error })))?;
    radarr.movie_snapshot.store(movies.clone()).await;
    Ok(Json(movies))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DirectRadarrMovie {
    title: String,
    title_slug: Option<String>,
    year: Option<i64>,
    monitored: bool,
    has_file: bool,
    size_on_disk: u64,
    quality_profile: Option<String>,
    added: Option<String>,
    tmdb_id: Option<i64>,
    artwork_source: Option<String>,
    genres: Vec<String>,
}

async fn radarr_movies_live(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<DirectRadarrMovie>>, (StatusCode, Json<ApiError>)> {
    let radarr = state.radarr.read().await.clone();
    let Some(radarr) = radarr else {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ApiError { error: "Radarr is not configured" }),
        ));
    };
    let movies: Vec<serde_json::Value> = radarr
        .http
        .get(format!("{}/api/v3/movie", radarr.base_url))
        .header("X-Api-Key", radarr.api_key.clone())
        .header(ACCEPT, "application/json")
        .send().await
        .map_err(|_| (StatusCode::BAD_GATEWAY, Json(ApiError { error: "Radarr movie library request failed" })))?
        .error_for_status()
        .map_err(|_| (StatusCode::BAD_GATEWAY, Json(ApiError { error: "Radarr movie library request failed" })))?
        .json().await
        .map_err(|_| (StatusCode::BAD_GATEWAY, Json(ApiError { error: "Radarr movie library response was invalid" })))?;
    radarr.movie_snapshot.store_poster_movies(movies.clone()).await;
    let profiles_response = radarr
        .http
        .get(format!("{}/api/v3/qualityprofile", radarr.base_url))
        .header("X-Api-Key", radarr.api_key.clone())
        .header(ACCEPT, "application/json")
        .send().await;
    let profiles: Vec<serde_json::Value> = match profiles_response {
        Ok(response) => response.json().await.unwrap_or_default(),
        Err(_) => Vec::new(),
    };
    Ok(Json(movies.into_iter().map(|movie| {
        let tmdb_id = movie.get("tmdbId").and_then(|v| v.as_i64());
        let profile_id = movie.get("qualityProfileId").and_then(|v| v.as_i64());
        let quality_profile = profiles.iter()
            .find(|profile| profile.get("id").and_then(|v| v.as_i64()) == profile_id)
            .and_then(|profile| profile.get("name")).and_then(|v| v.as_str())
            .or_else(|| movie.get("qualityProfile").and_then(|v| v.get("name")).and_then(|v| v.as_str()))
            .map(str::to_owned);
        let has_poster = movie.get("images").and_then(serde_json::Value::as_array).is_some_and(|images| images.iter().any(|image| {
            image.get("coverType").and_then(|v| v.as_str()).is_some_and(|kind| kind.eq_ignore_ascii_case("poster"))
        }));
        DirectRadarrMovie {
            title: movie.get("title").and_then(|v| v.as_str()).unwrap_or("Unknown title").to_owned(),
            title_slug: movie.get("titleSlug").and_then(|v| v.as_str()).map(str::to_owned),
            year: movie.get("year").and_then(|v| v.as_i64()),
            monitored: movie.get("monitored").and_then(|v| v.as_bool()).unwrap_or(false),
            has_file: movie.get("hasFile").and_then(|v| v.as_bool()).unwrap_or(false),
            size_on_disk: movie.get("sizeOnDisk").and_then(|v| v.as_u64()).unwrap_or_default(),
            quality_profile,
            added: movie.get("added").and_then(|v| v.as_str()).map(str::to_owned),
            tmdb_id,
            artwork_source: (has_poster && tmdb_id.is_some()).then(|| "radarr".to_owned()),
            genres: movie.get("genres").and_then(serde_json::Value::as_array).into_iter().flatten().filter_map(serde_json::Value::as_str).map(str::to_owned).collect(),
        }
    }).collect()))
}

async fn radarr_movie_poster_live(
    State(state): State<Arc<AppState>>,
    AxumPath(tmdb_id): AxumPath<u64>,
) -> Result<Response<Body>, (StatusCode, Json<ApiError>)> {
    if tmdb_id == 0 {
        return Err((StatusCode::BAD_REQUEST, Json(ApiError { error: "Invalid movie ID" })));
    }
    let radarr = state.radarr.read().await.clone().ok_or((
        StatusCode::SERVICE_UNAVAILABLE,
        Json(ApiError { error: "Radarr is not configured" }),
    ))?;
    let movie = radarr.movie_by_tmdb_id(tmdb_id).await
        .map_err(|_| (StatusCode::BAD_GATEWAY, Json(ApiError { error: "Radarr poster request failed" })))?
        .ok_or((StatusCode::NOT_FOUND, Json(ApiError { error: "Movie poster was not found" })))?;
    let (content_type, bytes) = radarr.fetch_movie_poster(&movie).await
        .map_err(|_| (StatusCode::BAD_GATEWAY, Json(ApiError { error: "Radarr poster request failed" })))?
        .ok_or((StatusCode::NOT_FOUND, Json(ApiError { error: "Movie poster was not found" })))?;
    let mut response = image_response(content_type, bytes)?;
    response.headers_mut().insert(header::CACHE_CONTROL, header::HeaderValue::from_static("private, no-store"));
    Ok(response)
}

async fn radarr_movie_poster(
    State(state): State<Arc<AppState>>,
    AxumPath(tmdb_id): AxumPath<u64>,
) -> Result<Response<Body>, (StatusCode, Json<ApiError>)> {
    if tmdb_id == 0 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                error: "Invalid movie ID",
            }),
        ));
    }
    let vault = state.vault.read().await.clone();
    if let Some(vault) = &vault {
        let id = tmdb_id;
        match tokio::task::spawn_blocking({
            let vault = vault.clone();
            let cache = state.artwork_cache.clone();
            move || vault.load_artwork_cached(id, &cache)
        })
        .await
        {
            Ok(Ok(Some(artwork))) => {
                return image_response(artwork.content_type, artwork.bytes);
            }
            Ok(Ok(None)) => {}
            _ => {}
        }
    }

    let radarr = state.radarr.read().await.clone().ok_or((
        StatusCode::SERVICE_UNAVAILABLE,
        Json(ApiError {
            error: "Radarr is not configured",
        }),
    ))?;
    let _guard = state.artwork_scrape_lock.lock().await;
    let (content_type, bytes) = scrape_and_store_movie_artwork(&radarr, vault, tmdb_id, state.artwork_cache.clone(), false)
        .await
        .map_err(|_| {
            (
                StatusCode::BAD_GATEWAY,
                Json(ApiError {
                    error: "Could not retrieve or save Radarr poster",
                }),
            )
        })?
        .ok_or((
            StatusCode::NOT_FOUND,
            Json(ApiError {
                error: "Movie poster was not found",
            }),
        ))?;
    image_response(content_type, bytes)
}

async fn scrape_and_store_movie_artwork(
    radarr: &RadarrClient,
    vault: Option<secure_config::AgeVault>,
    tmdb_id: u64,
    artwork_cache: Arc<artwork_cache::ArtworkCache>,
    refresh_metadata: bool,
) -> Result<Option<(String, Vec<u8>)>, ()> {
    let Some(movie) = radarr.movie_by_tmdb_id(tmdb_id).await.map_err(|_| ())? else {
        return Ok(None);
    };
    let vault_ref = vault.as_ref().ok_or(())?;
    if refresh_metadata || !vault_ref.movie_metadata_exists(tmdb_id) {
        let metadata = metadata::arr_text_record("radarr", "tmdbId", tmdb_id, &movie);
        let serialized = serde_json::to_vec(&metadata).map_err(|_| ())?;
        let vault = vault_ref.clone();
        tokio::task::spawn_blocking(move || vault.store_movie_metadata(tmdb_id, &serialized))
            .await.map_err(|_| ())?.map_err(|_| ())?;
    }
    if vault_ref.artwork_exists(tmdb_id) {
        let cached_vault = vault_ref.clone();
        let cache = artwork_cache;
        let artwork = tokio::task::spawn_blocking(move || cached_vault.load_artwork_cached(tmdb_id, &cache))
            .await.map_err(|_| ())?.map_err(|_| ())?;
        return Ok(artwork.map(|artwork| (artwork.content_type, artwork.bytes)));
    }
    let Some((content_type, bytes)) = radarr.fetch_movie_poster(&movie).await.map_err(|_| ())? else {
        return Ok(None);
    };
    let store_vault = vault_ref.clone();
    let store_type = content_type.clone();
    let store_bytes = bytes.clone();
    tokio::task::spawn_blocking(move || store_vault.store_artwork(tmdb_id, &store_type, &store_bytes))
        .await.map_err(|_| ())?.map_err(|_| ())?;
    Ok(Some((content_type, bytes)))
}

async fn metadata_artwork_backfill_status(
    State(state): State<Arc<AppState>>,
) -> Json<ArtworkBackfillProgress> {
    Json(state.artwork_backfill.read().await.clone())
}

async fn metadata_sonarr_backfill_status(
    State(state): State<Arc<AppState>>,
) -> Json<ArtworkBackfillProgress> {
    Json(state.sonarr_backfill.read().await.clone())
}

async fn request_movie_metadata_rescan(State(state): State<Arc<AppState>>) -> Response {
    state.artwork_force_rescan.store(true, Ordering::Release);
    state.artwork_scrape_notify.notify_one();
    (StatusCode::ACCEPTED, Json(serde_json::json!({"accepted": true}))).into_response()
}

async fn request_tv_metadata_rescan(State(state): State<Arc<AppState>>) -> Response {
    state.sonarr_scrape_notify.notify_one();
    (StatusCode::ACCEPTED, Json(serde_json::json!({"accepted": true}))).into_response()
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MetadataStorageStatus {
    bytes: u64,
    movies_bytes: u64,
    tv_bytes: u64,
}

async fn metadata_storage_status() -> Json<MetadataStorageStatus> {
    let metadata_dir = secure_config::AgeVault::metadata_dir();
    let bytes = tokio::task::spawn_blocking(move || {
        fn directory_size(path: &std::path::Path) -> u64 {
            let Ok(entries) = std::fs::read_dir(path) else {
                return 0;
            };
            entries
                .filter_map(Result::ok)
                .map(|entry| {
                    let Ok(file_type) = entry.file_type() else {
                        return 0;
                    };
                    if file_type.is_dir() {
                        directory_size(&entry.path())
                    } else if file_type.is_file() {
                        entry.metadata().map(|metadata| metadata.len()).unwrap_or(0)
                    } else {
                        0
                    }
                })
                .sum()
        }
        let movies_bytes = directory_size(&metadata_dir.join("movies"));
        let tv_bytes = directory_size(&metadata_dir.join("tv"));
        (directory_size(&metadata_dir), movies_bytes, tv_bytes)
    })
    .await
    .unwrap_or((0, 0, 0));
    Json(MetadataStorageStatus { bytes: bytes.0, movies_bytes: bytes.1, tv_bytes: bytes.2 })
}

async fn run_metadata_artwork_backfill(state: Arc<AppState>) {
    wait_for_metadata_backfill(&state, Duration::from_secs(10)).await;
    loop {
        let radarr = state.radarr.read().await.clone();
        let Some(radarr) = radarr else {
            wait_for_metadata_backfill(&state, Duration::from_secs(60)).await;
            continue;
        };
        let vault = state.vault.read().await.clone();
        let Some(migration_vault) = vault.clone() else {
            wait_for_metadata_backfill(&state, Duration::from_secs(60)).await;
            continue;
        };
        if !matches!(tokio::task::spawn_blocking(move || migration_vault.migrate_legacy_movie_filenames()).await, Ok(Ok(_))) {
            wait_for_metadata_backfill(&state, Duration::from_secs(60)).await;
            continue;
        }
        let movies = match {
            let _guard = state.artwork_scrape_lock.lock().await;
            radarr.list_movies(vault.clone()).await
        } {
            Ok(movies) => movies,
            Err(_) => {
                wait_for_metadata_backfill(&state, Duration::from_secs(60)).await;
                continue;
            }
        };
        let total = movies.iter().filter(|movie| movie.poster_url.is_some()).count();
        let scraped = movies.iter()
            .filter(|movie| movie.poster_url.is_some() && movie.artwork_source.as_deref() == Some("shared"))
            .count();
        let force_rescan = state.artwork_force_rescan.swap(false, Ordering::AcqRel);
        let missing: Vec<u64> = movies.iter()
            .filter(|movie| force_rescan || (movie.poster_url.is_some() && movie.artwork_source.as_deref() == Some("radarr")))
            .filter_map(|movie| movie.tmdb_id.and_then(|id| u64::try_from(id).ok()))
            .collect();
        if missing.is_empty() {
            *state.artwork_backfill.write().await = ArtworkBackfillProgress {
                active: false,
                completed: 0,
                scraped,
                total,
                failed: 0,
            };
            wait_for_metadata_backfill(&state, Duration::from_secs(6 * 60 * 60)).await;
            continue;
        }
        *state.artwork_backfill.write().await = ArtworkBackfillProgress {
            active: true,
            completed: 0,
            scraped,
            total,
            failed: 0,
        };
        for tmdb_id in missing {
            let cached = vault.as_ref().is_some_and(|vault| vault.artwork_exists(tmdb_id));
            let succeeded = if cached && !force_rescan { true } else {
                let _guard = state.artwork_scrape_lock.lock().await;
                matches!(scrape_and_store_movie_artwork(&radarr, vault.clone(), tmdb_id, state.artwork_cache.clone(), force_rescan).await, Ok(Some(_)))
                    || (force_rescan && vault.as_ref().is_some_and(|vault| vault.movie_metadata_exists(tmdb_id)))
            };
            let mut progress = state.artwork_backfill.write().await;
            progress.completed += 1;
            if succeeded {
                if !cached { progress.scraped += 1; }
            } else { progress.failed += 1; }
            drop(progress);
            // Yield between items and keep requests gentle on small Arr hosts.
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
        state.artwork_backfill.write().await.active = false;
        wait_for_metadata_backfill(&state, Duration::from_secs(6 * 60 * 60)).await;
    }
}

async fn wait_for_metadata_backfill(state: &AppState, delay: Duration) {
    tokio::select! {
        _ = tokio::time::sleep(delay) => {},
        _ = state.artwork_scrape_notify.notified() => {},
    }
}

async fn wait_for_sonarr_backfill(state: &AppState, delay: Duration) {
    tokio::select! {
        _ = tokio::time::sleep(delay) => {},
        _ = state.sonarr_scrape_notify.notified() => {},
    }
}

async fn run_metadata_sonarr_backfill(state: Arc<AppState>) {
    wait_for_sonarr_backfill(&state, Duration::from_secs(10)).await;
    loop {
        let vault = state.vault.read().await.clone();
        let Some(vault) = vault else {
            wait_for_sonarr_backfill(&state, Duration::from_secs(60)).await;
            continue;
        };
        let client = match sonarr::client(&state).await {
            Ok(client) => client,
            Err(_) => {
                wait_for_sonarr_backfill(&state, Duration::from_secs(60)).await;
                continue;
            }
        };
        let (series, profiles) = match {
            let _guard = state.artwork_scrape_lock.lock().await;
            match sonarr::raw_series(&client).await {
                Ok(series) => sonarr::raw_quality_profiles(&client).await.map(|profiles| (series, profiles)),
                Err(error) => Err(error),
            }
        } {
            Ok(result) => result,
            Err(_) => {
                wait_for_sonarr_backfill(&state, Duration::from_secs(60)).await;
                continue;
            }
        };
        let records: Vec<_> = series.into_iter().filter_map(|mut item| {
            let tvdb_id = item.get("tvdbId").and_then(serde_json::Value::as_u64).filter(|id| *id > 0)?;
            let quality_profile_id = item.get("qualityProfileId").and_then(serde_json::Value::as_i64);
            if let Some(profile) = profiles.iter().find(|profile| {
                profile.get("id").and_then(serde_json::Value::as_i64) == quality_profile_id
            }).and_then(|profile| profile.get("name")) {
                item["qualityProfile"] = profile.clone();
            }
            Some((tvdb_id, item))
        }).collect();
        let total = records.iter().filter(|(_, item)| sonarr::has_poster(item)).count();
        let scraped = records.iter().filter(|(tvdb_id, item)| {
            sonarr::has_poster(item) && vault.tv_artwork_exists(*tvdb_id)
        }).count();
        *state.sonarr_backfill.write().await = ArtworkBackfillProgress {
            active: !records.is_empty(), completed: 0, scraped, total, failed: 0,
        };
        for (tvdb_id, item) in records {
            let cached = vault.tv_artwork_exists(tvdb_id);
            let expected_artwork = sonarr::has_poster(&item);
            let scraped = {
                let _guard = state.artwork_scrape_lock.lock().await;
                sonarr::scrape_and_store(&client, vault.clone(), &item, tvdb_id).await
            };
            let scrape_ok = match scraped {
                Ok(_) => true,
                Err(reason) => {
                    eprintln!("Sonarr metadata scrape failed: {reason}");
                    false
                }
            };
            let text_saved = vault.tv_metadata_exists(tvdb_id);
            let artwork_saved = vault.tv_artwork_exists(tvdb_id);
            let succeeded = scrape_ok && text_saved && (!expected_artwork || artwork_saved);
            let mut progress = state.sonarr_backfill.write().await;
            progress.completed += 1;
            if succeeded {
                if expected_artwork && !cached && artwork_saved { progress.scraped += 1; }
            } else {
                progress.failed += 1;
            }
            drop(progress);
            // Yield between items and keep requests gentle on small Arr hosts.
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
        state.sonarr_backfill.write().await.active = false;
        wait_for_sonarr_backfill(&state, Duration::from_secs(6 * 60 * 60)).await;
    }
}

fn image_response(
    content_type: String,
    bytes: Vec<u8>,
) -> Result<Response<Body>, (StatusCode, Json<ApiError>)> {
    Response::builder()
        .header(header::CONTENT_TYPE, content_type)
        .header(header::CACHE_CONTROL, "private, max-age=3600")
        .body(Body::from(bytes))
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError {
                    error: "Could not serve movie poster",
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
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|_| ())?;
    let mut response = client.get(url).send().await.map_err(|_| ())?;
    read_image_response(&mut response).await
}

async fn read_image_response(
    response: &mut reqwest::Response,
) -> Result<Option<(String, Vec<u8>)>, ()> {
    const MAX_ARTWORK_BYTES: usize = 8 * 1024 * 1024;
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
    let cache = state.artwork_cache.clone();
    let artwork = tokio::task::spawn_blocking(move || vault.load_artwork_cached(tmdb_id, &cache))
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

async fn get_sonarr_poster(
    State(state): State<Arc<AppState>>,
    AxumPath(tvdb_id): AxumPath<u64>,
) -> Result<Response<Body>, (StatusCode, Json<ApiError>)> {
    let vault = state.vault.read().await.clone().ok_or((
        StatusCode::LOCKED,
        Json(ApiError { error: "Unlock the age-encrypted config to read cached artwork" }),
    ))?;
    let cache = state.artwork_cache.clone();
    let artwork = tokio::task::spawn_blocking(move || vault.load_tv_artwork_cached(tvdb_id, &cache))
        .await
        .map_err(|_| {
            eprintln!("TV artwork worker failed while loading encrypted artwork");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiError { error: "Could not decrypt cached artwork" }))
        })?
        .map_err(|error| {
            eprintln!("TV artwork read/decrypt failed: {error}");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiError { error: "Could not decrypt cached artwork" }))
        })?
        .ok_or((StatusCode::NOT_FOUND, Json(ApiError { error: "Encrypted artwork was not found" })))?;
    Response::builder()
        .header(header::CONTENT_TYPE, artwork.content_type)
        .header(header::CACHE_CONTROL, "private, no-store")
        .body(Body::from(artwork.bytes))
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiError { error: "Could not serve cached artwork" })))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MiniMetadataExportIndex {
    text_files: Vec<String>,
    artwork_files: Vec<String>,
}

pub(crate) fn valid_export_filename(kind: &str, filename: &str) -> bool {
    let stem = match kind {
        "text" => filename.strip_suffix(".json.aged"),
        "artwork" => filename.strip_suffix(".aged"),
        _ => None,
    }.unwrap_or("");
    !stem.is_empty()
        && (stem.bytes().all(|byte| byte.is_ascii_digit())
            || (stem.len() == 64 && stem.bytes().all(|byte| byte.is_ascii_hexdigit())))
}

fn list_export_files(kind: &str) -> Result<Vec<String>, std::io::Error> {
    let directory = secure_config::AgeVault::metadata_dir().join("movies").join(kind);
    let entries = match std::fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error),
    };
    let mut names = Vec::new();
    for entry in entries {
        let entry = entry?;
        if !entry.file_type()?.is_file() { continue; }
        let Some(name) = entry.file_name().to_str().map(str::to_owned) else { continue; };
        if valid_export_filename(kind, &name) { names.push(name); }
    }
    names.sort();
    Ok(names)
}

async fn mini_metadata_export_index() -> Result<Json<MiniMetadataExportIndex>, StatusCode> {
    let text_files = tokio::task::spawn_blocking(|| list_export_files("text"))
        .await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let artwork_files = tokio::task::spawn_blocking(|| list_export_files("artwork"))
        .await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(MiniMetadataExportIndex { text_files, artwork_files }))
}

async fn encrypted_radarr_library(
    State(state): State<Arc<AppState>>,
    body: axum::body::Bytes,
) -> Result<Response<Body>, StatusCode> {
    if body.len() != 65 || body[0] != 4 {
        return Err(StatusCode::BAD_REQUEST);
    }
    let client_key = p256::PublicKey::from_sec1_bytes(&body)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let radarr = state.radarr.read().await.clone().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let vault = state.vault.read().await.clone();
    let movies = radarr.list_movies(vault).await.map_err(|_| StatusCode::BAD_GATEWAY)?;
    let browser_movies: Vec<_> = movies.iter().map(|movie| BrowserRadarrMovie {
        title: &movie.title,
        title_slug: &movie.title_slug,
        year: movie.year,
        monitored: movie.monitored,
        has_file: movie.has_file,
        size_on_disk: movie.size_on_disk,
        quality_profile: &movie.quality_profile,
        added: &movie.added,
        tmdb_id: movie.tmdb_id,
        artwork_source: &movie.artwork_source,
        genres: movie.genres.iter().map(String::as_str).collect(),
    }).collect();
    let plaintext = Zeroizing::new(serde_json::to_vec(&browser_movies).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?);
    if plaintext.len() > 8 * 1024 * 1024 { return Err(StatusCode::PAYLOAD_TOO_LARGE); }
    let ephemeral = p256::ecdh::EphemeralSecret::random(&mut p256::elliptic_curve::rand_core::OsRng);
    let server_key = p256::PublicKey::from(&ephemeral);
    let shared = ephemeral.diffie_hellman(&client_key);
    let hkdf = hkdf::Hkdf::<sha2::Sha256>::new(
        Some(b"rawrarr-coordinator-session-v1"),
        shared.raw_secret_bytes(),
    );
    let mut aes_key = Zeroizing::new([0u8; 32]);
    hkdf.expand(b"radarr-library-response", &mut *aes_key)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let cipher = aes_gcm::Aes256Gcm::new_from_slice(&*aes_key).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let mut iv = [0u8; 12];
    p256::elliptic_curve::rand_core::OsRng.fill_bytes(&mut iv);
    use aes_gcm::aead::Aead;
    let encrypted = cipher.encrypt((&iv).into(), plaintext.as_slice()).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let mut response = Vec::with_capacity(65 + iv.len() + encrypted.len());
    response.extend_from_slice(server_key.to_encoded_point(false).as_bytes());
    response.extend_from_slice(&iv);
    response.extend_from_slice(&encrypted);
    Response::builder().header(header::CONTENT_TYPE, "application/octet-stream").header(header::CACHE_CONTROL, "private, no-store")
        .body(Body::from(response)).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BrowserRadarrMovie<'a> {
    title: &'a str,
    title_slug: &'a Option<String>,
    year: Option<i64>,
    monitored: bool,
    has_file: bool,
    size_on_disk: u64,
    quality_profile: &'a Option<String>,
    added: &'a Option<String>,
    tmdb_id: Option<i64>,
    artwork_source: &'a Option<String>,
    genres: Vec<&'a str>,
}

async fn export_ciphertext(
    AxumPath((kind, filename)): AxumPath<(String, String)>,
) -> Result<Response<Body>, StatusCode> {
    if !valid_export_filename(&kind, &filename) { return Err(StatusCode::BAD_REQUEST); }
    let path = secure_config::AgeVault::metadata_dir().join("movies").join(&kind).join(&filename);
    let bytes = tokio::task::spawn_blocking(move || {
        let metadata = std::fs::symlink_metadata(&path)?;
        let max_bytes = if kind == "artwork" { 10 * 1024 * 1024 } else { 7 * 1024 * 1024 + 4096 };
        if !metadata.file_type().is_file() || metadata.len() > max_bytes {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "unsupported export entry"));
        }
        std::fs::read(path)
    }).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .map_err(|error| if error.kind() == std::io::ErrorKind::NotFound { StatusCode::NOT_FOUND } else { StatusCode::INTERNAL_SERVER_ERROR })?;
    Response::builder()
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .header(header::CACHE_CONTROL, "private, no-store")
        .body(Body::from(bytes))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn export_config_ciphertext() -> Result<Response<Body>, StatusCode> {
    let path = secure_config::AgeVault::config_path();
    let bytes = tokio::task::spawn_blocking(move || {
        let metadata = std::fs::symlink_metadata(&path)?;
        if !metadata.file_type().is_file() || metadata.len() > 2 * 1024 * 1024 {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "unsupported config ciphertext"));
        }
        std::fs::read(path)
    })
        .await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .map_err(|error| if error.kind() == std::io::ErrorKind::NotFound { StatusCode::NOT_FOUND } else { StatusCode::INTERNAL_SERVER_ERROR })?;
    Response::builder()
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .header(header::CACHE_CONTROL, "private, no-store")
        .body(Body::from(bytes))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn export_age_recipient() -> Result<Response<Body>, StatusCode> {
    let path = secure_config::AgeVault::public_recipient_path();
    let recipient = tokio::task::spawn_blocking(move || std::fs::read_to_string(path))
        .await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .map_err(|error| if error.kind() == std::io::ErrorKind::NotFound { StatusCode::NOT_FOUND } else { StatusCode::INTERNAL_SERVER_ERROR })?;
    if !recipient.trim().starts_with("age1pq1") { return Err(StatusCode::SERVICE_UNAVAILABLE); }
    Response::builder().header(header::CONTENT_TYPE, "text/plain; charset=utf-8").header(header::CACHE_CONTROL, "private, no-store")
        .body(Body::from(recipient.trim().to_owned())).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn export_node_age_recipient() -> Result<Response<Body>, StatusCode> {
    let recipient = tokio::task::spawn_blocking(secure_config::AgeVault::node_identity_recipient)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    if !recipient.starts_with("age1pq1") {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    }
    Response::builder()
        .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
        .header(header::CACHE_CONTROL, "private, no-store")
        .body(Body::from(recipient))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn replace_config_ciphertext(body: Bytes) -> Result<StatusCode, StatusCode> {
    const MAGIC: &[u8] = b"age-encryption.org/v1\n";
    if body.len() < MAGIC.len() || body.len() > 2 * 1024 * 1024 || !body.starts_with(MAGIC) {
        return Err(StatusCode::BAD_REQUEST);
    }
    let path = secure_config::AgeVault::config_path();
    tokio::task::spawn_blocking(move || {
        use std::io::Write;
        let parent = path.parent().ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
        std::fs::create_dir_all(parent).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let temporary = parent.join(format!(".config.{}.tmp", uuid::Uuid::new_v4().simple()));
        let mut file = std::fs::OpenOptions::new().write(true).create_new(true).open(&temporary).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let result = (|| {
            file.write_all(&body).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            file.sync_all().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            std::fs::rename(&temporary, &path).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
        })();
        if result.is_err() { let _ = std::fs::remove_file(&temporary); }
        result
    }).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)??;
    Ok(StatusCode::NO_CONTENT)
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
    let vault = state.vault.read().await;
    Json(AgeSettingsInfo {
        unlocked: vault.is_some(),
        encrypted_config_exists: secure_config::AgeVault::config_path().is_file(),
        theme_seed: vault
            .as_ref()
            .map(|vault| age_theme_seed(vault.recipient())),
    })
}

fn age_theme_seed(recipient: &str) -> String {
    Sha256::digest(recipient.as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
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
            metadata_filename_key_configured: stored_config
                .as_ref()
                .is_some_and(|config| config.metadata_filename_key.is_some()),
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
struct SyncthingSharedHealthResponse {
    configured: bool,
    folders: Option<Vec<syncthing::SharedHealth>>,
}

async fn syncthing_shared_health(
    State(state): State<Arc<AppState>>,
) -> Json<SyncthingSharedHealthResponse> {
    let Some(client) = &state.syncthing else {
        return Json(SyncthingSharedHealthResponse { configured: false, folders: None });
    };
    match client.shared_health_all().await {
        Ok(folders) => Json(SyncthingSharedHealthResponse { configured: true, folders: Some(folders) }),
        Err(_) => Json(SyncthingSharedHealthResponse { configured: true, folders: None }),
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
        theme_seed: age_theme_seed(&recipient),
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
    if request.remember_on_device {
        vault.persist_identity_locally().map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError {
                    error: "Could not save the local age identity",
                }),
            )
        })?;
    }
    vault.persist_public_recipient().map_err(|error| {
        let (status, message) = match error {
            secure_config::VaultError::InvalidIdentity => (
                StatusCode::CONFLICT,
                "This shared folder already belongs to another age ID. Paste the original private identity to unlock it; creating a new ID does not recover existing encrypted data.",
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Could not save the public age recipient for storage nodes; check shared-folder write permissions",
            ),
        };
        (status, Json(ApiError { error: message }))
    })?;
    if let Some(settings) = stored.filter(|settings| !settings.radarr_url.is_empty()) {
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
    let theme_seed = age_theme_seed(vault.recipient());
    *state.vault.write().await = Some(vault);
    Ok(Json(AgeSettingsInfo {
        unlocked: true,
        encrypted_config_exists: exists,
        theme_seed: Some(theme_seed),
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
                RadarrConfigError::InvalidUrl => "Enter a valid Radarr HTTPS URL",
                RadarrConfigError::InvalidApiKey => "Enter a valid Radarr API key",
            };
            (StatusCode::BAD_REQUEST, Json(ApiError { error: message }))
        })?;
    let status = client.status().await.map_err(|_| {
        (
            StatusCode::BAD_GATEWAY,
            Json(ApiError {
                error: "Could not connect to Radarr; check the URL and API key",
            }),
        )
    })?;
    if !status.app_name.eq_ignore_ascii_case("radarr") {
        return Err((
            StatusCode::BAD_GATEWAY,
            Json(ApiError {
                error: "The configured service did not identify itself as Radarr",
            }),
        ));
    }
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
            device_names: std::collections::HashMap::new(),
            sonarr: None,
            radarr_url: url.clone(),
            radarr_api_key: api_key.to_owned(),
            metadata_filename_key: None,
            additional_services: Vec::new(),
            needs_migration: false,
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
    let role = node_role::NodeRole::from_env()?;
    let media_configured = env::var("RAWARR_LIBRARY_PATH").is_ok_and(|p| !p.trim().is_empty());
    role.validate_media(media_configured)?;
    let arguments: Vec<String> = env::args().skip(1).collect();
    if !arguments.is_empty()
        && (arguments != ["storage-scan-once"] || role != node_role::NodeRole::Storage)
    {
        return Err(
            "only storage supports the storage-scan-once command; other arguments are invalid"
                .into(),
        );
    }
    if role == node_role::NodeRole::Coordinator {
        let address = env::var("RUST_ARR_LISTEN_ADDR").unwrap_or_else(|_| "127.0.0.1:8080".into());
        let app = coordinator::router().await?;
        let listener = tokio::net::TcpListener::bind(&address).await?;
        println!("rawrarr coordinator listening on {address}");
        axum::serve(listener, app).await?;
        return Ok(());
    }
    secure_config::AgeVault::ensure_node_identity()?;
    secure_config::AgeVault::ensure_node_signing_key()?;
    let device_id = devices::load_or_create_device_id()?;
    let device_name = env::var("RAWARR_DEVICE_NAME").unwrap_or_else(|_| "This device".to_owned());
    let node_id = env::var("RAWARR_NODE_ID").unwrap_or_else(|_| device_id.clone());
    let auth = node_auth::NodeAuth::from_env(role, &node_id, &device_name).await?;
    let role_name = role.as_str();
    if arguments == ["storage-scan-once"] {
        let logger = secure_config::AgeVault::node_signing_public_key().ok().and_then(|public_key| {
            shared_logs::SharedLogger::open(&secure_config::AgeVault::shared_dir(), &device_id, &node_id, role_name, auth.signing_certificate(), public_key, |message| {
                secure_config::AgeVault::sign_node_message(message).map_err(std::io::Error::other)
            }).ok()
        });
        if logger.is_none() {
            eprintln!("rawrarr shared logging could not be initialized");
        }
        let summary = storage_node::scan_once()?;
        if let Some(logger) = logger {
            logger.scan_completed(
                summary.items.len(),
                summary.sidecars_created,
                summary.sidecar_errors,
                summary.issues,
                summary.truncated,
            );
        }
        println!(
            "Storage scan complete: {} media files; {} encrypted sidecars saved; {} already complete; {} sidecar errors; {} scan issues.",
            summary.items.len(),
            summary.sidecars_created,
            summary.sidecars_skipped,
            summary.sidecar_errors,
            summary.issues
        );
        if summary.sidecar_errors > 0 || summary.issues > 0 || summary.truncated {
            return Err("storage scan completed with errors or truncation".into());
        }
        return Ok(());
    }
    let address = env::var("RUST_ARR_LISTEN_ADDR").unwrap_or_else(|_| "127.0.0.1:8080".to_owned());
    let socket_address: SocketAddr = address.parse()?;
    let mut radarr = None;
    let mut vault = None;
    secure_config::AgeVault::enforce_local_config_policy()?;
    let local_settings = secure_config::AgeVault::load_local_config()?;
    let identity_path = env::var_os("RAWARR_AGE_IDENTITY_FILE")
        .filter(|path| !path.is_empty())
        .or_else(|| {
            let local_identity = env::var_os("RAWARR_DATA_DIR")
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|| std::path::PathBuf::from("data"))
                .join("age-identity");
            local_identity
                .is_file()
                .then(|| local_identity.into_os_string())
        });
    if let Some(identity_path) = identity_path {
        let identity = std::fs::read_to_string(identity_path)?;
        let unlocked = secure_config::AgeVault::from_identity(identity)?;
        if let Some(settings) = unlocked
            .load()?
            .or(local_settings)
            .filter(|settings| !settings.radarr_url.is_empty())
        {
            radarr = Some(RadarrClient::new(
                settings.radarr_url.clone(),
                settings.radarr_api_key.clone(),
            )?);
        }
        vault = Some(unlocked);
    } else if let Some(settings) = local_settings {
        radarr = Some(RadarrClient::new(
            settings.radarr_url.clone(),
            settings.radarr_api_key.clone(),
        )?);
    }
    let shared_logs = secure_config::AgeVault::node_signing_public_key().ok().and_then(|public_key| {
        shared_logs::SharedLogger::open(&secure_config::AgeVault::shared_dir(), &device_id, &node_id, role_name, auth.signing_certificate(), public_key, |message| {
            secure_config::AgeVault::sign_node_message(message).map_err(std::io::Error::other)
        }).ok()
    });
    if shared_logs.is_none() {
        eprintln!("rawrarr shared logging could not be initialized");
    }
    let storage_scan = storage_node::ScanController::default();
    if role == node_role::NodeRole::Storage {
        if let Some(interval) = storage_node::scan_interval_from_env()? {
            storage_node::start_periodic_scan_with_controller(
                interval,
                shared_logs.clone(),
                storage_scan.clone(),
            );
            println!(
                "storage automatic scanning enabled; scans repeat every {} seconds",
                interval.as_secs()
            );
        } else {
            println!("storage automatic scanning disabled by configuration");
        }
    }
    let state = Arc::new(AppState {
        auth: auth.clone(),
        radarr: Arc::new(RwLock::new(radarr)),
        vault: Arc::new(RwLock::new(vault)),
        library: library::LibraryRoot::from_env()?,
        device_id,
        shared_logs: shared_logs.clone(),
        role,
        storage_scan,
        syncthing: syncthing::Client::from_env()
            .map_err(|_| "invalid Syncthing environment configuration")?,
        artwork_backfill: Arc::new(RwLock::new(ArtworkBackfillProgress::default())),
        sonarr_backfill: Arc::new(RwLock::new(ArtworkBackfillProgress::default())),
        artwork_scrape_lock: Arc::new(tokio::sync::Mutex::new(())),
        artwork_scrape_notify: Arc::new(tokio::sync::Notify::new()),
        artwork_force_rescan: Arc::new(AtomicBool::new(false)),
        sonarr_scrape_notify: Arc::new(tokio::sync::Notify::new()),
        artwork_cache: Arc::new(artwork_cache::ArtworkCache::from_env()),
    });
    if role == node_role::NodeRole::Metadata {
        tokio::spawn(run_metadata_artwork_backfill(state.clone()));
        tokio::spawn(run_metadata_sonarr_backfill(state.clone()));
    }

    let protected = Router::new()
        .route("/api/auth/logout", post(logout))
        .route("/api/events", get(dashboard_events))
        .route("/api/library", get(library_info))
        .route("/api/devices", get(list_devices))
        .route("/api/nodes/pending", get(node_auth::proxy))
        .route("/api/nodes/approve", post(node_auth::proxy))
        .route("/api/syncthing/status", get(syncthing_status))
        .route("/api/syncthing/shared-health", get(syncthing_shared_health))
        .route("/api/syncthing/share", post(configure_syncthing_share))
        .route("/api/library/scan", axum::routing::post(scan_library))
        .route("/api/library/scan/missing", post(request_storage_scan))
        .route("/api/library/scan/status", get(storage_scan_status))
        .route("/api/library/scan/cancel", post(cancel_storage_scan))
        .route(
            "/api/metadata/movies/search",
            axum::routing::post(search_movie_metadata),
        )
        .route(
            "/api/metadata/movies/{tmdb_id}/poster",
            get(get_movie_poster),
        )
        .route("/api/metadata/status", get(metadata_status))
        .route("/api/metadata/export/mini", get(mini_metadata_export_index))
        .route("/api/metadata/session/library", axum::routing::post(encrypted_radarr_library))
        .route("/api/metadata/export/config", get(export_config_ciphertext).put(replace_config_ciphertext))
        .route("/api/metadata/export/recipient", get(export_age_recipient))
        .route("/api/metadata/export/node-recipient", get(export_node_age_recipient))
        .route("/api/metadata/export/ciphertext/{kind}/{filename}", get(export_ciphertext))
        .route("/api/metadata/storage", get(metadata_storage_status))
        .route("/api/metadata/artwork/backfill", get(metadata_artwork_backfill_status))
        .route("/api/metadata/artwork/rescan", post(request_movie_metadata_rescan))
        .route("/api/metadata/sonarr/backfill", get(metadata_sonarr_backfill_status))
        .route("/api/metadata/sonarr/rescan", post(request_tv_metadata_rescan))
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
        .route("/api/sonarr/status", get(sonarr::status))
        .route("/api/sonarr/series", get(sonarr::series))
        .route("/api/sonarr/series/{tvdb_id}/poster", get(get_sonarr_poster))
        .route(
            "/api/settings/sonarr",
            get(sonarr::settings)
                .put(sonarr::save)
                .delete(sonarr::clear),
        )
        .route("/api/radarr/status", get(radarr_status))
        .route("/api/radarr/movies", get(radarr_movies))
        .route("/api/radarr/movies/snapshot", get(movie_snapshot::get))
        .route(
            "/api/radarr/movies/{tmdb_id}/poster",
            get(radarr_movie_poster),
        )
        .route_layer(middleware::from_fn_with_state(
            auth.clone(),
            node_auth::require_session,
        ));
    let direct_protected = Router::new()
        .route("/api/library/scan/missing", post(request_storage_scan))
        .route("/api/library/scan/status", get(storage_scan_status))
        .route("/api/library/scan/cancel", post(cancel_storage_scan))
        .route("/api/metadata/storage", get(metadata_storage_status))
        .route("/api/metadata/sonarr/backfill", get(metadata_sonarr_backfill_status))
        .route("/api/metadata/sonarr/rescan", post(request_tv_metadata_rescan))
        .route("/api/metadata/artwork/rescan", post(request_movie_metadata_rescan))
        .route("/api/metadata/export/mini", get(mini_metadata_export_index))
        .route("/api/metadata/session/library", post(encrypted_radarr_library))
        .route("/api/metadata/export/config", get(export_config_ciphertext).put(replace_config_ciphertext))
        .route("/api/metadata/export/recipient", get(export_age_recipient))
        .route("/api/metadata/export/node-recipient", get(export_node_age_recipient))
        .route("/api/metadata/export/ciphertext/{kind}/{filename}", get(export_ciphertext))
        .route("/api/settings/age", get(get_age_settings).post(unlock_age_settings))
        .route(
            "/api/settings/radarr",
            get(get_radarr_settings)
                .put(put_radarr_settings)
                .delete(delete_radarr_settings),
        )
        .route(
            "/api/settings/sonarr",
            get(sonarr::settings)
                .put(sonarr::save)
                .delete(sonarr::clear),
        )
        .route("/api/sonarr/status", get(sonarr::status))
        .route("/api/sonarr/series", get(sonarr::series))
        .route("/api/sonarr/series/{tvdb_id}/poster", get(get_sonarr_poster))
        .route("/api/radarr/movies", get(radarr_movies))
        .route("/api/radarr/live-library", get(radarr_movies_live))
        .route("/api/radarr/live-movies/{tmdb_id}/poster", get(radarr_movie_poster_live))
        .route_layer(middleware::from_fn_with_state(
            auth.clone(),
            node_auth::require_session,
        ));
    let direct_app = Router::new()
        .route("/api/direct/health", get(direct_health))
        .route("/api/direct/session", post(direct_session_cookie))
        .merge(direct_protected)
        .layer(middleware::from_fn(log_request))
        .layer(middleware::from_fn(direct_api_cors))
        .with_state(state.clone());
    let app = Router::new()
        .route("/health", get(health))
        .route("/api/nodes/enroll", post(node_auth::enrollment_proxy))
        .route(
            "/api/nodes/enroll/status",
            post(node_auth::enrollment_proxy),
        )
        .route("/assets/rawrarr-logo.png", get(rawrarr_logo))
        .route("/assets/rawrarr-logo.jxl", get(rawrarr_logo_jxl))
        .route("/assets/rawrarr-favicon.svg", get(rawrarr_favicon))
        .route("/assets/passkey-client.mjs", get(passkey_client_js))
        .route("/assets/metadata-export.js", get(metadata_export_js))
        .route("/api/auth/status", get(node_auth::proxy))
        .route("/api/auth/register/start", post(node_auth::proxy))
        .route("/api/auth/register/finish", post(node_auth::proxy))
        .route("/api/auth/login/start", post(node_auth::proxy))
        .route("/api/auth/login/finish", post(node_auth::proxy))
        .route(
            "/api/auth/passkeys/coordinator/link",
            post(node_auth::proxy),
        )
        .merge(protected);
    let app = if role == node_role::NodeRole::Coordinator {
        app.route("/", get(dashboard))
            .route("/auth", get(auth_page))
            .route("/setup/age", get(age_setup_page))
    } else {
        app.route("/", get(subscriber_ui_disabled))
            .route("/auth", get(subscriber_ui_disabled))
            .route("/setup/age", get(subscriber_ui_disabled))
    }
    .layer(middleware::from_fn(log_request))
    .with_state(state);
    let listener = tokio::net::TcpListener::bind(socket_address).await?;
    if let Ok(direct_address) = env::var("RUST_ARR_DIRECT_LISTEN_ADDR") {
        if !matches!(role, node_role::NodeRole::Metadata | node_role::NodeRole::Storage) {
            return Err("the direct API listener is available only for metadata or storage nodes".into());
        }
        let direct_socket_address: SocketAddr = direct_address.parse()?;
        let direct_listener = tokio::net::TcpListener::bind(direct_socket_address).await?;
        tokio::spawn(async move {
            if let Err(error) = axum::serve(direct_listener, direct_app).await {
                eprintln!("rawrarr direct API listener failed: {error}");
            }
        });
        println!("rawrarr direct metadata API listening on {direct_socket_address}");
    }

    if let Some(logger) = shared_logs {
        logger.application_started();
    }
    if quiet_logs_enabled(env::var("RAWARR_QUIET_LOGS").ok().as_deref()) {
        println!("rawrarr request logging is quiet; server errors remain visible");
    } else {
        println!("rawrarr verbose request logging enabled");
    }
    println!("rawrarr listening on {socket_address}");
    axum::serve(listener, app).await?;
    Ok(())
}

async fn direct_session_cookie(headers: HeaderMap) -> Response {
    let token = headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .filter(|value| value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit()));
    let Some(token) = token else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    let cookie = format!("rawrarr_direct_session={token}; Path=/direct-api/; HttpOnly; Secure; SameSite=Strict; Max-Age=300");
    match HeaderValue::from_str(&cookie) {
        Ok(value) => Response::builder()
            .status(StatusCode::NO_CONTENT)
            .header(header::SET_COOKIE, value)
            .body(Body::empty())
            .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response()),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

async fn direct_api_cors(request: Request, next: Next) -> Response {
    let origin = request.headers().get(header::ORIGIN).cloned();
    let allowed_origin = env::var("RAWARR_DIRECT_ALLOWED_ORIGIN").ok();
    let allowed = origin
        .as_ref()
        .and_then(|value| value.to_str().ok())
        .zip(allowed_origin.as_deref())
        .is_some_and(|(request_origin, allowed)| request_origin == allowed);
    if request.method() == axum::http::Method::OPTIONS {
        let mut response = StatusCode::NO_CONTENT.into_response();
        if allowed {
            add_direct_cors_headers(&mut response, origin.as_ref());
        }
        return response;
    }
    let mut response = next.run(request).await;
    if allowed {
        add_direct_cors_headers(&mut response, origin.as_ref());
    }
    response
}

fn add_direct_cors_headers(response: &mut Response, origin: Option<&HeaderValue>) {
    let Some(origin) = origin else { return };
    let headers = response.headers_mut();
    headers.insert(header::VARY, HeaderValue::from_static("Origin"));
    headers.insert("access-control-allow-origin", origin.clone());
    headers.insert("access-control-allow-credentials", HeaderValue::from_static("true"));
    headers.insert("access-control-allow-methods", HeaderValue::from_static("GET, POST, PUT, DELETE, OPTIONS"));
    headers.insert("access-control-allow-headers", HeaderValue::from_static("Accept, Authorization, Content-Type"));
}

async fn subscriber_ui_disabled() -> StatusCode {
    StatusCode::NOT_FOUND
}

#[cfg(test)]
mod metadata_image_tests {
    use super::{ArrClient, validated_tmdb_image_url};

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

    #[test]
    fn arr_connections_require_https() {
        assert!(ArrClient::new("https://radarr.example:7878".into(), "test-key".into()).is_ok());
        assert!(ArrClient::new("https://sonarr.example:8989".into(), "test-key".into()).is_ok());
        for url in ["http://radarr.example:7878", "http://sonarr.example:8989", "ftp://arr.example"] {
            assert!(
                ArrClient::new(url.into(), "test-key".into()).is_err(),
                "accepted {url}"
            );
        }
    }
}

#[cfg(test)]
mod logging_tests {
    use super::quiet_logs_enabled;

    #[test]
    fn quiet_logging_is_opt_in_and_accepts_common_boolean_values() {
        assert!(!quiet_logs_enabled(None));
        assert!(!quiet_logs_enabled(Some("false")));
        for value in ["true", "TRUE", "1", "yes", "on"] {
            assert!(
                quiet_logs_enabled(Some(value)),
                "expected {value} to enable quiet logging"
            );
        }
    }
}

#[cfg(test)]
mod age_identity_request_tests {
    use super::AgeIdentityRequest;

    #[test]
    fn browser_camel_case_remember_flag_is_deserialized() {
        let request: AgeIdentityRequest = serde_json::from_str(
            r#"{"identity":"AGE-SECRET-KEY-PQ-1fixture","rememberOnDevice":true}"#,
        )
        .expect("decode age unlock request");
        assert!(request.remember_on_device);
    }
}

#[cfg(test)]
mod metadata_export_tests {
    use super::valid_export_filename;

    #[test]
    fn export_names_accept_only_legacy_ids_or_opaque_hmac_names() {
        assert!(valid_export_filename("text", "123.json.aged"));
        assert!(valid_export_filename("artwork", "123.aged"));
        assert!(valid_export_filename("text", &format!("{}.json.aged", "a".repeat(64))));
        assert!(valid_export_filename("artwork", &format!("{}.aged", "f".repeat(64))));
        for name in ["../secret.aged", "x.json.aged", "a/b.aged", &format!("{}.aged", "g".repeat(64))] {
            assert!(!valid_export_filename("artwork", name));
        }
        assert!(!valid_export_filename("unknown", "123.aged"));
    }
}
