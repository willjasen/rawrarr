use super::*;

#[tokio::test]
async fn snapshots_are_shared_by_clones_but_not_replacement_clients() {
    let cache = Cache::default();
    assert!(cache.snapshot.read().await.is_none());
    cache.store(Vec::new()).await;
    let clone = cache.clone();
    assert!(clone.snapshot.read().await.as_ref().unwrap().movies.is_empty());
    assert!(clone.snapshot.read().await.as_ref().unwrap().updated_at > 0);
    // Client replacement on connection save/clear/sign-out cannot inherit old data.
    assert!(Cache::default().snapshot.read().await.is_none());
}

#[tokio::test]
async fn direct_library_poster_records_are_cached_by_tmdb_id() {
    let cache = Cache::default();
    cache.store_poster_movies(vec![
        serde_json::json!({"tmdbId": 42, "images": [{"coverType": "poster", "url": "/poster.jpg"}]}),
        serde_json::json!({"tmdbId": 0}),
    ]).await;
    let movie = cache.poster_movie(42).await.unwrap();
    assert_eq!(movie["images"][0]["url"], "/poster.jpg");
    assert!(cache.poster_movie(99).await.is_none());
}
