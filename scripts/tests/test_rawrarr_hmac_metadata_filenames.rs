use super::*;

#[test]
fn movie_files_use_domain_separated_hmac_names_and_migrate_ciphertext() {
    let root = std::env::temp_dir().join(format!("rawrarr-hmac-filenames-{}", uuid::Uuid::new_v4()));
    let shared = root.join("shared");
    let identity = Command::new("age-keygen").arg("-pq").output().unwrap();
    assert!(identity.status.success());
    let identity = String::from_utf8(identity.stdout).unwrap();
    let vault = AgeVault::from_identity_in_dirs(identity.clone(), shared.clone(), root.join("local/config.json"), false).unwrap();
    vault.save(&PersistedConfig {
        device_names: std::collections::HashMap::new(),
        sonarr: None,
        radarr_url: "https://radarr.example".into(),
        radarr_api_key: "fixture".into(),
        radarr_ui_username: String::new(),
        radarr_ui_password: String::new(),
        metadata_filename_key: None,
        additional_services: Vec::new(),
        needs_migration: false,
    }).unwrap();

    let text_path = vault.opaque_path_for("text", 12345).unwrap();
    let art_path = vault.opaque_path_for("artwork", 12345).unwrap();
    let text_name = text_path.file_name().unwrap().to_string_lossy();
    let art_name = art_path.file_name().unwrap().to_string_lossy();
    assert_eq!(text_name.len(), 64 + ".json.aged".len());
    assert_eq!(art_name.len(), 64 + ".aged".len());
    assert_ne!(&text_name[..64], &art_name[..64]);
    assert!(text_name[..64].bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase()));
    assert!(art_name[..64].bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase()));

    let legacy_text = vault.metadata_dir.join("movies/text/12345.json.aged");
    let legacy_art = vault.metadata_dir.join("movies/artwork/12345.aged");
    let metadata = br#"{"schemaVersion":1,"tmdbId":12345}"#;
    encrypt_shared_file(vault.recipient(), &legacy_text, metadata).unwrap();
    store_artwork_with_recipient(vault.recipient(), 12345, "image/jpeg", b"fixture", &legacy_art).unwrap();
    let old_text_bytes = fs::read(&legacy_text).unwrap();
    let old_art_bytes = fs::read(&legacy_art).unwrap();

    assert_eq!(vault.migrate_legacy_movie_filenames().unwrap(), 2);
    assert!(!legacy_text.exists());
    assert!(!legacy_art.exists());
    assert_eq!(fs::read(&text_path).unwrap(), old_text_bytes);
    assert_eq!(fs::read(&art_path).unwrap(), old_art_bytes);
    assert_eq!(vault.load_movie_metadata(12345).unwrap().unwrap()["tmdbId"], 12345);
    assert_eq!(vault.load_artwork(12345).unwrap().unwrap().bytes, b"fixture");
    fs::remove_dir_all(root).unwrap();
}
