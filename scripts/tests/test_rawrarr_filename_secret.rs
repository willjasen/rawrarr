use super::*;

fn fixture() -> PersistedConfig {
    PersistedConfig {
        device_names: std::collections::HashMap::new(),
        sonarr: None,
        radarr_url: "https://radarr.example".into(),
        radarr_api_key: "disposable-fixture".into(),
        radarr_ui_username: String::new(),
        radarr_ui_password: String::new(),
        metadata_filename_key: None,
        additional_services: Vec::new(),
        needs_migration: false,
    }
}

#[test]
fn filename_key_is_random_valid_and_stable() {
    let mut first = fixture();
    let mut second = fixture();
    assert!(ensure_filename_key(&mut first).unwrap());
    assert!(ensure_filename_key(&mut second).unwrap());
    assert!(first.metadata_filename_key != second.metadata_filename_key);
    let original = first.metadata_filename_key.clone();
    assert!(!ensure_filename_key(&mut first).unwrap());
    assert!(first.metadata_filename_key == original);
    for invalid in ["", "abc", &"g".repeat(64), &"a".repeat(63)] {
        first.metadata_filename_key = Some(invalid.into());
        assert!(matches!(
            ensure_filename_key(&mut first),
            Err(VaultError::InvalidConfig)
        ));
    }
}

#[test]
fn filename_key_survives_encrypted_config_lifecycle() {
    let root = std::env::temp_dir().join(format!("rawrarr-filename-key-{}", uuid::Uuid::new_v4()));
    let shared = root.join("shared");
    let local = root.join("local/radarr-config.json");
    let output = Command::new("age-keygen").arg("-pq").output().unwrap();
    assert!(output.status.success());
    let identity = String::from_utf8(output.stdout).unwrap();
    let vault =
        AgeVault::from_identity_in_dirs(identity.clone(), shared.clone(), local.clone(), true)
            .unwrap();
    let peer = AgeVault::from_identity_in_dirs(
        identity,
        shared.clone(),
        root.join("peer/radarr-config.json"),
        false,
    )
    .unwrap();
    let mut config = fixture();
    // Write a real legacy encrypted config without a key, then migrate on unlock/load.
    vault.write_config(&config).unwrap();
    let loaded = vault.load().unwrap().unwrap();
    let key = loaded.metadata_filename_key.as_ref().unwrap();
    let ciphertext = fs::read(shared.join("config.aged")).unwrap();
    assert!(!ciphertext.windows(key.len()).any(|w| w == key.as_bytes()));
    let mirror: serde_json::Value = serde_json::from_slice(&fs::read(&local).unwrap()).unwrap();
    assert!(mirror.get("metadata_filename_key").is_none());
    assert!(peer.load().unwrap().unwrap().metadata_filename_key.as_ref() == Some(key));
    assert!(fs::read(shared.join("config.aged")).unwrap() == ciphertext);
    config.radarr_api_key = "changed-fixture".into();
    config.metadata_filename_key = Some("b".repeat(64));
    vault.save(&config).unwrap();
    assert!(
        vault
            .load()
            .unwrap()
            .unwrap()
            .metadata_filename_key
            .as_ref()
            == Some(key)
    );
    vault.clear().unwrap();
    let cleared = vault.load().unwrap().unwrap();
    assert!(cleared.radarr_url.is_empty() && cleared.radarr_api_key.is_empty());
    assert!(cleared.metadata_filename_key.as_ref() == Some(key));
    assert!(!local.exists());
    config.metadata_filename_key = None;
    vault.save(&config).unwrap();
    assert!(
        vault
            .load()
            .unwrap()
            .unwrap()
            .metadata_filename_key
            .as_ref()
            == Some(key)
    );
    fs::remove_dir_all(root).unwrap();
}
