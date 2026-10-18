use super::*;

#[test]
fn tv_metadata_and_artwork_use_separate_opaque_age_encrypted_paths() {
    let keygen = Command::new("age-keygen").arg("-pq").output().unwrap();
    assert!(keygen.status.success());
    let unique = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let root = env::temp_dir().join(format!("rawrarr-sonarr-metadata-{unique}"));
    let vault = AgeVault::from_identity_in_dirs(
        String::from_utf8(keygen.stdout).unwrap(), root.join("shared"), root.join("local.json"), false,
    ).unwrap();
    vault.save(&PersistedConfig {
        device_names: std::collections::HashMap::new(),
        sonarr: None,
        radarr_url: String::new(),
        radarr_api_key: String::new(),
        additional_services: Vec::new(),
        needs_migration: false,
        metadata_filename_key: None,
    }).unwrap();

    let series = br#"{"schemaVersion":1,"provider":"sonarr","tvdbId":123456,"title":"Fixture Series"}"#;
    vault.store_tv_metadata(123456, series).unwrap();
    vault.store_tv_artwork(123456, "image/jpeg", b"fixture poster bytes").unwrap();

    let metadata = vault.tv_metadata_path(123456).unwrap();
    let artwork = vault.tv_artwork_path(123456).unwrap();
    assert!(vault.tv_metadata_exists(123456));
    assert!(vault.tv_artwork_exists(123456));
    assert!(metadata.starts_with(root.join("shared/metadata/tv/text")));
    assert!(artwork.starts_with(root.join("shared/metadata/tv/artwork")));
    assert!(!metadata.to_string_lossy().contains("123456"));
    assert!(!artwork.to_string_lossy().contains("123456"));
    assert_ne!(metadata.file_name(), artwork.file_name());
    assert_ne!(fs::read(metadata).unwrap(), series);
    assert_ne!(fs::read(artwork).unwrap(), b"image/jpeg\nfixture poster bytes");
    let decrypted = vault.load_tv_artwork(123456).unwrap().unwrap();
    assert_eq!(decrypted.content_type, "image/jpeg");
    assert_eq!(decrypted.bytes, b"fixture poster bytes");
    fs::remove_dir_all(root).unwrap();
}
