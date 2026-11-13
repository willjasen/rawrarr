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

#[cfg(unix)]
#[test]
fn artwork_cache_rejects_symlinked_ciphertext() {
    use std::os::unix::fs::symlink;

    let target = temporary_file("symlink-target", b"age-ciphertext");
    let path = target.with_extension("link");
    symlink(&target, &path).unwrap();
    assert!(ArtworkCache::disabled().read(&path).is_err());
    assert!(ArtworkCache::from_options(true, Some("redis://127.0.0.1:1/"), 86_400)
        .read(&path)
        .is_err());
    fs::remove_file(path).unwrap();
    fs::remove_file(target).unwrap();
}

#[test]
fn valkey_stores_and_returns_encrypted_artwork_when_configured() {
    let Ok(url) = std::env::var("RAWARR_TEST_VALKEY_URL") else {
        eprintln!("SKIP: set RAWARR_TEST_VALKEY_URL to run the live Valkey artwork-cache integration check");
        return;
    };
    let cache = ArtworkCache::from_options(true, Some(&url), 86_400);
    assert!(cache.is_configured(), "Valkey URL must be accepted by the Redis-compatible client");
    assert!(cache.ping(), "Valkey must answer PING");

    let client = redis::Client::open(url).unwrap();
    let mut connection = client.get_connection().unwrap();
    let before: Vec<String> = redis::cmd("KEYS").arg("rawrarr:artwork:v1:*").query(&mut connection).unwrap();
    let ciphertext = b"age-encrypted-artwork-fixture";
    let path = temporary_file("valkey", ciphertext);
    assert_eq!(cache.read(&path).unwrap(), ciphertext);

    let keys: Vec<String> = redis::cmd("KEYS").arg("rawrarr:artwork:v1:*").query(&mut connection).unwrap();
    let added: Vec<&String> = keys.iter().filter(|key| !before.contains(key)).collect();
    assert_eq!(added.len(), 1, "a cache miss should store one new versioned artwork entry");
    let stored: Vec<u8> = redis::cmd("GET").arg(added[0]).query(&mut connection).unwrap();
    assert_eq!(stored, ciphertext, "Valkey must contain the encrypted bytes unchanged");

    assert_eq!(cache.read(&path).unwrap(), ciphertext, "repeat read should return the Valkey value");
    fs::remove_file(path).unwrap();
}
