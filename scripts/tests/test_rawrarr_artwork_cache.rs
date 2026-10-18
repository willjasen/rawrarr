use super::ArtworkCache;
use std::{fs, path::PathBuf, time::{SystemTime, UNIX_EPOCH}};

fn temporary_file(name: &str, bytes: &[u8]) -> PathBuf {
    let nonce = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let path = std::env::temp_dir().join(format!("rawrarr-artwork-cache-{name}-{}-{nonce}", std::process::id()));
    fs::write(&path, bytes).unwrap();
    path
}

#[test]
fn disabled_cache_reads_authoritative_file_and_observes_replacement() {
    let path = temporary_file("disabled", b"encrypted-first");
    let cache = ArtworkCache::disabled();
    assert_eq!(cache.read(&path).unwrap(), b"encrypted-first");
    fs::write(&path, b"encrypted-second").unwrap();
    assert_eq!(cache.read(&path).unwrap(), b"encrypted-second");
    fs::remove_file(path).unwrap();
}

#[test]
fn enabled_but_unreachable_cache_falls_back_to_disk() {
    let cache = ArtworkCache::from_options(true, Some("redis://127.0.0.1:1/"), 86_400);
    let path = temporary_file("fallback", b"age-ciphertext");
    assert_eq!(cache.read(&path).unwrap(), b"age-ciphertext");
    fs::remove_file(path).unwrap();
}
