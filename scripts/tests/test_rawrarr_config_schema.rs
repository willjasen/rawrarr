use super::*;

#[test]
fn legacy_flat_radarr_and_sonarr_config_reads_and_serializes_as_schema_v1() {
    let legacy: PersistedConfig = serde_json::from_str(r#"{
      "device_names":{"node-a":"Living room"},
      "radarr_url":"https://radarr.example/",
      "radarr_api_key":"radarr-secret",
      "sonarr":{"url":"https://sonarr.example/","api_key":"sonarr-secret"},
      "metadata_filename_key":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
    }"#).unwrap();
    assert!(legacy.needs_migration);
    assert_eq!(legacy.radarr_url, "https://radarr.example/");
    assert_eq!(legacy.sonarr.as_ref().unwrap().url, "https://sonarr.example/");

    let canonical = serde_json::to_value(&legacy).unwrap();
    assert_eq!(canonical["schema_version"], 1);
    assert_eq!(canonical["services"][0]["type"], "radarr");
    assert_eq!(canonical["services"][0]["api_key"], "radarr-secret");
    assert_eq!(canonical["services"][1]["type"], "sonarr");
    assert!(canonical.get("radarr_url").is_none());
    assert!(canonical.get("radarr_api_key").is_none());
    assert!(canonical.get("sonarr").is_none());

    let parsed: PersistedConfig = serde_json::from_value(canonical).unwrap();
    assert!(!parsed.needs_migration);
    assert_eq!(parsed.radarr_api_key, "radarr-secret");
}

#[test]
fn unsupported_config_schema_versions_are_rejected() {
    let result = serde_json::from_str::<PersistedConfig>(r#"{"schema_version":2,"services":[]}"#);
    assert!(result.is_err());
}

#[test]
fn encrypted_radarr_ui_credentials_round_trip_in_canonical_service_config() {
    let config: PersistedConfig = serde_json::from_str(
        r#"{
          "schema_version":1,
          "device_names":{},
          "services":[
            {
              "type":"radarr",
              "url":"https://radarr.example/",
              "api_key":"fixture-api-key",
              "ui_username":"fixture-ui-user",
              "ui_password":"fixture-ui-password"
            }
          ],
          "metadata_filename_key":null
        }"#,
    )
    .unwrap();
    assert!(!config.needs_migration);
    assert_eq!(config.radarr_ui_username, "fixture-ui-user");
    assert_eq!(config.radarr_ui_password, "fixture-ui-password");

    let serialized = serde_json::to_value(&config).unwrap();
    assert_eq!(
        serialized["services"][0]["ui_username"],
        "fixture-ui-user"
    );
    assert_eq!(
        serialized["services"][0]["ui_password"],
        "fixture-ui-password"
    );
}

#[test]
fn unlocking_legacy_ciphertext_rewrites_it_as_canonical_schema_v1() {
    let root = std::env::temp_dir().join(format!("rawrarr-config-migration-{}", uuid::Uuid::new_v4()));
    let shared = root.join("shared");
    std::fs::create_dir_all(&shared).unwrap();
    let output = Command::new("age-keygen").arg("-pq").output().unwrap();
    assert!(output.status.success());
    let identity = String::from_utf8(output.stdout).unwrap();
    let vault = AgeVault::from_identity_in_shared_dir(identity, shared.clone()).unwrap();
    let legacy = br#"{"radarr_url":"https://radarr.test/","radarr_api_key":"fixture-secret"}"#;
    encrypt_shared_file(vault.recipient(), &shared.join("config.aged"), legacy).unwrap();

    let config = vault.load().unwrap().unwrap();
    assert_eq!(config.radarr_url, "https://radarr.test/");
    assert!(!config.needs_migration);
    let reloaded = vault.read_config().unwrap().unwrap();
    assert!(!reloaded.needs_migration);
    assert_eq!(reloaded.radarr_api_key, "fixture-secret");
    std::fs::remove_dir_all(root).unwrap();
}
