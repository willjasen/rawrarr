use super::*;
use std::time::{SystemTime, UNIX_EPOCH};

fn fixture_root(name: &str) -> PathBuf {
    env::temp_dir().join(format!(
        "rawrarr-metadata-{name}-{}-{}",
        std::process::id(),
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()
    ))
}

#[test]
fn fresh_shared_volume_initializes_metadata_without_legacy_files() {
    let root = fixture_root("fresh");
    fs::create_dir_all(&root).unwrap();
    migrate_legacy_metadata(&root).unwrap();
    let target = root.join("in/metadata");
    assert!(target.is_dir());
    assert!(!root.join("metadata").exists());
    fs::write(target.join("preserved.aged"), b"ciphertext").unwrap();
    migrate_legacy_metadata(&root).unwrap();
    assert_eq!(fs::read(target.join("preserved.aged")).unwrap(), b"ciphertext");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(fs::metadata(&target).unwrap().permissions().mode() & 0o777, 0o755);
    }
    fs::remove_dir_all(root).unwrap();
}

#[cfg(unix)]
#[test]
fn fresh_volume_repairs_container_alias_and_accepts_encrypted_writes() {
    let root = fixture_root("alias");
    let shared = root.join("shared");
    fs::create_dir_all(&shared).unwrap();
    let alias = root.join("metadata");
    std::os::unix::fs::symlink(shared.join("in/metadata"), &alias).unwrap();
    assert!(!alias.exists(), "reproduce the container's dangling alias");

    migrate_legacy_metadata(&shared).unwrap();
    let (_, recipient) = AgeVault::generate_identity().unwrap();
    let path = alias.join("movies/text/fixture.json.aged");
    encrypt_shared_file(&recipient, &path, br#"{"tmdbId":1,"title":"Fixture"}"#).unwrap();
    let ciphertext = fs::read(&path).unwrap();
    assert!(ciphertext.starts_with(b"age-encryption.org/v1\n"));
    assert!(!ciphertext.windows(b"Fixture".len()).any(|part| part == b"Fixture"));
    assert_eq!(crate::directory_size(&alias).unwrap(), ciphertext.len() as u64);
    migrate_legacy_metadata(&shared).unwrap();
    assert_eq!(fs::read(&path).unwrap(), ciphertext);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn metadata_initialization_rejects_files_without_replacing_them() {
    for relative in ["in", "in/metadata"] {
        let root = fixture_root("blocked");
        let blocked = root.join(relative);
        fs::create_dir_all(blocked.parent().unwrap()).unwrap();
        fs::write(&blocked, b"preserve").unwrap();
        assert!(matches!(migrate_legacy_metadata(&root), Err(VaultError::StorageFailed)));
        assert_eq!(fs::read(&blocked).unwrap(), b"preserve");
        fs::remove_dir_all(root).unwrap();
    }
}

#[cfg(unix)]
#[test]
fn metadata_initialization_rejects_symlinked_shared_subdirectories() {
    for relative in ["in", "in/metadata"] {
        let root = fixture_root("symlink");
        let shared = root.join("shared");
        let outside = root.join("outside");
        fs::create_dir_all(&outside).unwrap();
        let blocked = shared.join(relative);
        fs::create_dir_all(blocked.parent().unwrap()).unwrap();
        std::os::unix::fs::symlink(&outside, &blocked).unwrap();
        assert!(matches!(migrate_legacy_metadata(&shared), Err(VaultError::StorageFailed)));
        assert_eq!(fs::read_link(&blocked).unwrap(), outside);
        assert_eq!(fs::read_dir(&outside).unwrap().count(), 0);
        fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn storage_counts_nested_files_and_propagates_read_errors() {
    let root = fixture_root("sizes");
    assert_eq!(crate::directory_size(&root).unwrap(), 0);
    fs::create_dir_all(root.join("movies/text")).unwrap();
    fs::create_dir_all(root.join("tv/text")).unwrap();
    fs::write(root.join("movies/text/movie.aged"), b"movie").unwrap();
    fs::write(root.join("tv/text/series.aged"), b"series").unwrap();
    assert_eq!(crate::directory_size(&root).unwrap(), 11);
    assert_eq!(crate::directory_size(&root.join("movies")).unwrap(), 5);
    assert_eq!(crate::directory_size(&root.join("tv")).unwrap(), 6);
    assert!(crate::directory_size(&root.join("movies/text/movie.aged")).is_err());
    fs::remove_dir_all(root).unwrap();
}
