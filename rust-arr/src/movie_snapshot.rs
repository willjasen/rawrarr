//! Last successful list, scoped to the active Radarr client; never persisted.
use crate::{AppState, RadarrMovie};
use axum::{Json, extract::State};
use serde::Serialize;
use std::{
    collections::HashMap,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::sync::RwLock;

#[derive(Clone, Default)]
pub(crate) struct Cache {
    snapshot: Arc<RwLock<Option<Snapshot>>>,
    poster_movies: Arc<RwLock<HashMap<u64, serde_json::Value>>>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Snapshot {
    movies: Vec<RadarrMovie>,
    updated_at: u64,
}

impl Cache {
    pub(crate) async fn store(&self, movies: Vec<RadarrMovie>) {
        *self.snapshot.write().await = Some(Snapshot {
            movies,
            updated_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        });
    }

    pub(crate) async fn store_poster_movies(&self, movies: Vec<serde_json::Value>) {
        let indexed = movies.into_iter().filter_map(|movie| {
            let id = movie.get("tmdbId").and_then(serde_json::Value::as_u64)?;
            Some((id, movie))
        }).collect();
        *self.poster_movies.write().await = indexed;
    }

    pub(crate) async fn poster_movie(&self, tmdb_id: u64) -> Option<serde_json::Value> {
        self.poster_movies.read().await.get(&tmdb_id).cloned()
    }
}

pub(crate) async fn get(State(state): State<Arc<AppState>>) -> impl axum::response::IntoResponse {
    let client = state.radarr.read().await.clone();
    let snapshot = match client {
        Some(client) => client.movie_snapshot.snapshot.read().await.clone(),
        None => None,
    };
    (
        [(axum::http::header::CACHE_CONTROL, "no-store")],
        Json(snapshot),
    )
}

#[cfg(test)]
#[path = "../../scripts/tests/test_rawrarr_movie_snapshot.rs"]
mod tests;
