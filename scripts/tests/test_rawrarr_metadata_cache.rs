use super::*;
use serde_json::json;

#[test]
fn ciphertext_cache_reuses_unchanged_data_and_invalidates_replacements() {
    let path = std::env::temp_dir().join(format!("rawrarr-metadata-cache-{}", uuid::Uuid::new_v4()));
    fs::write(&path, b"first ciphertext").unwrap();
    let cache = MetadataCache::default();
    assert_eq!(cache.load(&path, || Ok(Some(json!({"name":"first"})))).unwrap(), Some(json!({"name":"first"})));
    assert_eq!(cache.load(&path, || panic!("unchanged metadata must not decrypt again")).unwrap(), Some(json!({"name":"first"})));
    // Same-length content change must invalidate, regardless of filesystem timestamps.
    fs::write(&path, b"other ciphertext").unwrap();
    assert_eq!(cache.load(&path, || Ok(Some(json!({"name":"other"})))).unwrap(), Some(json!({"name":"other"})));
    let another_vault = MetadataCache::default();
    assert!(another_vault.load(&path, || Err(VaultError::DecryptionFailed)).is_err());
    fs::remove_file(&path).unwrap();
    assert!(cache.load(&path, || panic!("removed file must not use cache")).is_err());
}

#[test]
fn changing_file_during_decryption_is_not_cached() {
    let path = std::env::temp_dir().join(format!("rawrarr-metadata-race-{}", uuid::Uuid::new_v4()));
    fs::write(&path, b"before").unwrap();
    let cache = MetadataCache::default();
    cache.load(&path, || { fs::write(&path, b"after").unwrap(); Ok(Some(json!("old"))) }).unwrap();
    assert_eq!(cache.load(&path, || Ok(Some(json!("new")))).unwrap(), Some(json!("new")));
    fs::remove_file(path).unwrap();
}
