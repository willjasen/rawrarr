use super::*;

#[tokio::test]
async fn snapshots_are_shared_by_clones_but_not_replacement_clients() {
    let cache = Cache::default();
    assert!(cache.0.read().await.is_none());
    cache.store(Vec::new()).await;
    let clone = cache.clone();
    assert!(clone.0.read().await.as_ref().unwrap().movies.is_empty());
    assert!(clone.0.read().await.as_ref().unwrap().updated_at > 0);
    // Client replacement on connection save/clear/sign-out cannot inherit old data.
    assert!(Cache::default().0.read().await.is_none());
}
