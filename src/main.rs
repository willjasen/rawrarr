#![forbid(unsafe_code)]

pub mod download_clients;
pub mod library;
pub mod metadata;

use axum::{Json, Router, extract::State, http::StatusCode, response::Html, routing::get};
use reqwest::header::{ACCEPT, HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use std::{env, net::SocketAddr, sync::Arc, time::Duration};
use tokio::sync::RwLock;

#[derive(Clone)]
struct AppState {
    radarr: Arc<RwLock<Option<RadarrClient>>>,
    library: Option<library::LibraryRoot>,
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
struct ApiError {
    error: &'static str,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RadarrSettingsRequest {
    url: String,
    api_key: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RadarrSettingsInfo {
    configured: bool,
    url: Option<String>,
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
}

async fn health() -> &'static str {
    "ok"
}

async fn dashboard() -> Html<&'static str> {
    Html(include_str!("ui.html"))
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
    provider
        .search_movies(request)
        .await
        .map(Json)
        .map_err(|error| {
            let (status, message) = match error {
                metadata::SearchError::InvalidQuery | metadata::SearchError::InvalidYear => {
                    (StatusCode::BAD_REQUEST, "Invalid movie search request")
                }
                metadata::SearchError::Upstream => {
                    (StatusCode::BAD_GATEWAY, "Movie metadata search failed")
                }
            };
            (status, Json(ApiError { error: message }))
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
    })
}

async fn put_radarr_settings(
    State(state): State<Arc<AppState>>,
    Json(settings): Json<RadarrSettingsRequest>,
) -> Result<Json<RadarrSettingsInfo>, (StatusCode, Json<ApiError>)> {
    let client = RadarrClient::new(settings.url, settings.api_key).map_err(|error| {
        let message = match error {
            RadarrConfigError::InvalidUrl => "Enter a valid Radarr HTTP or HTTPS URL",
            RadarrConfigError::InvalidApiKey => "Enter a valid Radarr API key",
        };
        (StatusCode::BAD_REQUEST, Json(ApiError { error: message }))
    })?;
    let url = client.base_url.clone();
    *state.radarr.write().await = Some(client);
    Ok(Json(RadarrSettingsInfo {
        configured: true,
        url: Some(url),
    }))
}

async fn delete_radarr_settings(State(state): State<Arc<AppState>>) -> Json<RadarrSettingsInfo> {
    *state.radarr.write().await = None;
    Json(RadarrSettingsInfo {
        configured: false,
        url: None,
    })
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address = env::var("RUST_ARR_LISTEN_ADDR").unwrap_or_else(|_| "127.0.0.1:8080".to_owned());
    let socket_address: SocketAddr = address.parse()?;
    let state = Arc::new(AppState {
        radarr: Arc::new(RwLock::new(RadarrClient::from_env()?)),
        library: library::LibraryRoot::from_env()?,
    });

    let app = Router::new()
        .route("/", get(dashboard))
        .route("/health", get(health))
        .route("/api/library", get(library_info))
        .route("/api/library/scan", axum::routing::post(scan_library))
        .route(
            "/api/metadata/movies/search",
            axum::routing::post(search_movie_metadata),
        )
        .route("/api/metadata/status", get(metadata_status))
        .route(
            "/api/settings/radarr",
            get(get_radarr_settings)
                .put(put_radarr_settings)
                .delete(delete_radarr_settings),
        )
        .route("/api/radarr/status", get(radarr_status))
        .with_state(state);
    let listener = tokio::net::TcpListener::bind(socket_address).await?;

    println!("rawrarr listening on {socket_address}");
    axum::serve(listener, app).await?;
    Ok(())
}
