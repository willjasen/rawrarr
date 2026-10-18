use super::*;
use std::{collections::HashMap, sync::{Arc, Mutex, atomic::{AtomicUsize, Ordering}}};

#[derive(Clone, Default)]
struct Cache(Arc<Mutex<HashMap<u64, Value>>>, bool);
impl MovieCache for Cache {
    fn load(&self, id: u64) -> Result<Option<Value>, &'static str> {
        Ok(self.0.lock().unwrap().get(&id).cloned())
    }
    fn store(&self, id: u64, value: &Value) -> Result<(), &'static str> {
        if self.1 { return Err("fixture storage failure"); }
        self.0.lock().unwrap().insert(id, value.clone());
        Ok(())
    }
}

async fn server(fail: bool) -> (RadarrClient, Arc<AtomicUsize>, tokio::task::JoinHandle<()>) {
    let requests = Arc::new(AtomicUsize::new(0));
    let counter = requests.clone();
    let app = axum::Router::new().route("/api/v3/qualityprofile", axum::routing::get(move || {
        let counter = counter.clone();
        async move {
            counter.fetch_add(1, Ordering::SeqCst);
            if fail { (axum::http::StatusCode::BAD_GATEWAY, axum::Json(json!({}))) }
            else { (axum::http::StatusCode::OK, axum::Json(json!([{"id":7,"name":"HD-1080p"},{"id":9,"name":"Ultra HD"}]))) }
        }
    }));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let client = RadarrClient { movie_snapshot: Default::default(), base_url: format!("http://{}", listener.local_addr().unwrap()),
        api_key: reqwest::header::HeaderValue::from_static("fixture"), http: reqwest::Client::new() };
    let task = tokio::spawn(async move { axum::serve(listener, app).await.unwrap(); });
    (client, requests, task)
}

#[tokio::test]
async fn cache_misses_are_batched_persisted_and_reused() {
    let (client, requests, task) = server(false).await;
    let cache = Cache::default();
    cache.0.lock().unwrap().insert(1, json!({"title":"Preserved title","overview":"Preserved overview","custom":{"keep":true}}));
    let movies = vec![json!({"tmdbId":1,"title":"Fixture","qualityProfileId":7}), json!({"tmdbId":2,"qualityProfileId":9})];
    let first = enrich_cached(&client, cache.clone(), movies.clone()).await.unwrap();
    assert_eq!(profile_name(&first[0]), Some("HD-1080p"));
    assert_eq!(profile_name(&first[1]), Some("Ultra HD"));
    assert_eq!(requests.load(Ordering::SeqCst), 1);
    let saved = cache.load(1).unwrap().unwrap();
    assert_eq!(saved["title"], "Preserved title");
    assert_eq!(saved["custom"]["keep"], true);
    assert_eq!(saved["qualityProfileId"], 7);
    task.abort(); // A warm cache must work with the profile endpoint offline.
    let second = enrich_cached(&client, cache, movies).await.unwrap();
    assert_eq!(profile_name(&second[0]), Some("HD-1080p"));
    assert_eq!(requests.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn cache_wins_over_embedded_name_and_changed_assignments_refresh() {
    let (client, requests, task) = server(false).await;
    let cache = Cache::default();
    cache.0.lock().unwrap().insert(1, json!({"qualityProfileId":7,"qualityProfile":{"name":"Shared name"}}));
    let first = enrich_cached(&client, cache.clone(), vec![json!({"tmdbId":1,"qualityProfileId":7,"qualityProfile":{"name":"Embedded name"}})]).await.unwrap();
    assert_eq!(profile_name(&first[0]), Some("Shared name"));
    assert_eq!(requests.load(Ordering::SeqCst), 0);
    let second = enrich_cached(&client, cache.clone(), vec![json!({"tmdbId":1,"qualityProfileId":9})]).await.unwrap();
    assert_eq!(profile_name(&second[0]), Some("Ultra HD"));
    assert_eq!(cache.load(1).unwrap().unwrap()["qualityProfileId"], 9);
    task.abort();
}

#[tokio::test]
async fn failures_are_explicit_and_unknown_profiles_are_not_cached() {
    let (client, _, task) = server(true).await;
    let movie = json!({"tmdbId":1,"qualityProfileId":7});
    assert_eq!(enrich_cached(&client, Cache::default(), vec![movie.clone()]).await.unwrap_err(), "Radarr quality profile request failed");
    task.abort();
    let (client, _, task) = server(false).await;
    assert_eq!(enrich_cached(&client, Cache(Arc::default(), true), vec![movie]).await.unwrap_err(), "fixture storage failure");
    let cache = Cache::default();
    let result = enrich_cached(&client, cache.clone(), vec![json!({"tmdbId":3,"qualityProfileId":999})]).await.unwrap();
    assert!(profile_name(&result[0]).is_none());
    assert!(cache.load(3).unwrap().is_none());
    task.abort();
}

#[test]
fn legacy_metadata_and_invalid_ids_are_safe() {
    let cache = Cache::default();
    cache.0.lock().unwrap().insert(1, json!({"qualityProfile":"Legacy name"}));
    let mut movies = vec![json!({"tmdbId":1,"qualityProfileId":7}), json!({"tmdbId":-1,"qualityProfileId":7})];
    let saved = read_cache(&mut movies, &cache).unwrap();
    assert_eq!(profile_name(&movies[0]), Some("Legacy name"));
    merge_profiles(&mut movies, &saved, &[json!({"id":7,"name":"HD"})], &cache).unwrap();
    assert_eq!(cache.0.lock().unwrap().len(), 1);
}

#[test]
fn parallel_cache_reads_preserve_movie_order_and_empty_libraries() {
    let cache = Cache::default();
    let mut movies: Vec<_> = (1..=33).map(|id| {
        cache.0.lock().unwrap().insert(id, json!({"qualityProfile":format!("Profile {id}")}));
        json!({"tmdbId":id})
    }).collect();
    let saved = read_cache(&mut movies, &cache).unwrap();
    for (index, movie) in movies.iter().enumerate() {
        let expected = format!("Profile {}", index + 1);
        assert_eq!(profile_name(movie), Some(expected.as_str()));
        assert_eq!(profile_name(saved[index].as_ref().unwrap()), Some(expected.as_str()));
    }
    assert!(read_cache(&mut [], &cache).unwrap().is_empty());
}
