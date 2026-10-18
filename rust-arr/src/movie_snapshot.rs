//! Last successful list, scoped to the active Radarr client; never persisted.
use crate::{AppState, RadarrMovie};
use axum::{extract::State, Json};
use serde::Serialize;
use std::{sync::Arc, time::{SystemTime, UNIX_EPOCH}};
use tokio::sync::RwLock;

#[derive(Clone, Default)]
pub(crate) struct Cache(Arc<RwLock<Option<Snapshot>>>);

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Snapshot {
    movies: Vec<RadarrMovie>,
    updated_at: u64,
}

impl Cache {
    pub(crate) async fn store(&self, movies: Vec<RadarrMovie>) {
        *self.0.write().await = Some(Snapshot {
            movies,
            updated_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs(),
        });
    }
}

pub(crate) async fn get(State(state): State<Arc<AppState>>) -> impl axum::response::IntoResponse {
    let client = state.radarr.read().await.clone();
    let snapshot = match client {
        Some(client) => client.movie_snapshot.0.read().await.clone(),
        None => None,
    };
    ([(axum::http::header::CACHE_CONTROL, "no-store")], Json(snapshot))
}

#[cfg(test)]
#[path = "../../scripts/tests/test_rawrarr_movie_snapshot.rs"]
mod tests;
