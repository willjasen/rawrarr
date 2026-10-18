use super::*;
#[test]
fn sonarr_and_radarr_updates_preserve_each_other_and_filename_key() {
    let output = Command::new("age-keygen").arg("-pq").output().unwrap();
    assert!(output.status.success());
    let root = env::temp_dir().join(format!(
        "rawrarr-arr-settings-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let vault = AgeVault::from_identity_in_dirs(
        String::from_utf8(output.stdout).unwrap(),
        root.join("shared"),
        root.join("local.json"),
        false,
    )
    .unwrap();
    let radarr = PersistedConfig {
        device_names: std::collections::HashMap::new(),
        radarr_url: "https://radarr.test".into(),
        radarr_api_key: "radarr-fixture".into(),
        sonarr: None,
        metadata_filename_key: None,
        additional_services: Vec::new(),
        needs_migration: false,
    };
    vault.save(&radarr).unwrap();
    let filename_key = vault.load().unwrap().unwrap().metadata_filename_key.clone();
    vault
        .save_sonarr(Some(ArrConnection {
            url: "https://sonarr.test".into(),
            api_key: "sonarr-fixture".into(),
        }))
        .unwrap();
    assert_eq!(
        vault.load().unwrap().unwrap().radarr_api_key,
        "radarr-fixture"
    );
    vault.save(&radarr).unwrap();
    assert_eq!(
        vault
            .load()
            .unwrap()
            .unwrap()
            .sonarr
            .as_ref()
            .unwrap()
            .api_key,
        "sonarr-fixture"
    );
    vault.clear().unwrap();
    let config = vault.load().unwrap().unwrap();
    assert!(config.radarr_api_key.is_empty());
    assert!(config.sonarr.is_some());
    assert_eq!(config.metadata_filename_key, filename_key);
    vault.save(&radarr).unwrap();
    vault.save_sonarr(None).unwrap();
    let config = vault.load().unwrap().unwrap();
    assert_eq!(config.radarr_api_key, "radarr-fixture");
    assert!(config.sonarr.is_none());
    assert_eq!(config.metadata_filename_key, filename_key);
    fs::remove_dir_all(root).unwrap();
}
