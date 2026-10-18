#![forbid(unsafe_code)]

pub mod download_clients;
pub mod library;

use axum::{Json, Router, extract::State, http::StatusCode, response::Html, routing::get};
use reqwest::header::{ACCEPT, HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use std::{env, net::SocketAddr, sync::Arc, time::Duration};

#[derive(Clone)]
struct AppState {
    radarr: Option<RadarrClient>,
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
        if api_key.is_empty() {
            return Err("RADARR_API_KEY must not be empty".into());
        }

        let mut api_key_header = HeaderValue::from_str(&api_key)?;
        api_key_header.set_sensitive(true);
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()?;

        Ok(Some(Self {
            base_url: base_url.trim_end_matches('/').to_owned(),
            api_key: api_key_header,
            http,
        }))
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
    let Some(radarr) = &state.radarr else {
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address = env::var("RUST_ARR_LISTEN_ADDR").unwrap_or_else(|_| "127.0.0.1:8080".to_owned());
    let socket_address: SocketAddr = address.parse()?;
    let state = Arc::new(AppState {
        radarr: RadarrClient::from_env()?,
        library: library::LibraryRoot::from_env()?,
    });

    let app = Router::new()
        .route("/", get(dashboard))
        .route("/health", get(health))
        .route("/api/library", get(library_info))
        .route("/api/library/scan", axum::routing::post(scan_library))
        .route("/api/radarr/status", get(radarr_status))
        .with_state(state);
    let listener = tokio::net::TcpListener::bind(socket_address).await?;

    println!("rawrarr listening on {socket_address}");
    axum::serve(listener, app).await?;
    Ok(())
}
