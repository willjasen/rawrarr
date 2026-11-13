use super::*;
use serde_json::json;
use std::fs;

#[test]
fn ciphertext_cache_reuses_unchanged_data_and_invalidates_replacements() {
    let path = std::env::temp_dir().join(format!("rawrarr-metadata-cache-{}", uuid::Uuid::new_v4()));
    fs::write(&path, b"first ciphertext").unwrap();
    let cache = MetadataCache::default();
    assert_eq!(
        cache
            .load(&path, 1024, || Ok(Some(json!({"name":"first"}))))
            .unwrap(),
        Some(json!({"name":"first"}))
    );
    assert_eq!(
        cache
            .load(&path, 1024, || panic!(
                "unchanged metadata must not decrypt again"
            ))
            .unwrap(),
        Some(json!({"name":"first"}))
    );
    // Same-length content change must invalidate, regardless of filesystem timestamps.
    fs::write(&path, b"other ciphertext").unwrap();
    assert_eq!(
        cache
            .load(&path, 1024, || Ok(Some(json!({"name":"other"}))))
            .unwrap(),
        Some(json!({"name":"other"}))
    );
    let another_vault = MetadataCache::default();
    assert!(
        another_vault
            .load(&path, 1024, || Err(VaultError::DecryptionFailed))
            .is_err()
    );
    fs::remove_file(&path).unwrap();
    assert_eq!(
        cache
            .load(&path, 1024, || panic!("removed file must not use cache"))
            .unwrap(),
        None
    );
}

#[test]
fn changing_file_during_decryption_is_not_cached() {
    let path = std::env::temp_dir().join(format!("rawrarr-metadata-race-{}", uuid::Uuid::new_v4()));
    fs::write(&path, b"before").unwrap();
    let cache = MetadataCache::default();
    cache
        .load(&path, 1024, || {
            fs::write(&path, b"after").unwrap();
            Ok(Some(json!("old")))
        })
        .unwrap();
    assert_eq!(
        cache.load(&path, 1024, || Ok(Some(json!("new")))).unwrap(),
        Some(json!("new"))
    );
    fs::remove_file(path).unwrap();
}

#[cfg(unix)]
#[test]
fn metadata_cache_rejects_symlinks_before_decryption() {
    use std::os::unix::fs::symlink;

    let root = std::env::temp_dir().join(format!("rawrarr-metadata-symlink-{}", uuid::Uuid::new_v4()));
    fs::create_dir(&root).unwrap();
    let target = root.join("target");
    let path = root.join("metadata.aged");
    fs::write(&target, b"ciphertext").unwrap();
    symlink(&target, &path).unwrap();

    assert!(
        MetadataCache::default()
            .load(&path, 1024, || panic!("symlink must be rejected before decryption"))
            .is_err()
    );
    fs::remove_dir_all(root).unwrap();
}
