use super::*;
use std::os::unix::fs::PermissionsExt;

#[tokio::test]
async fn owner_recipient_endpoint_returns_only_the_user_public_recipient() {
    let (identity, user_recipient) = AgeVault::generate_identity().unwrap();
    let group_recipient = "age1pq1group000000000000000000000";
    let vault = AgeVault {
        movie_metadata_cache: std::sync::Arc::default(),
        identity,
        user_recipient: user_recipient.clone(),
        group_identity: "synthetic group identity must not be returned".to_owned(),
        recipient: group_recipient.to_owned(),
        shared_dir: PathBuf::new(),
        metadata_dir: PathBuf::new(),
        config_path: PathBuf::new(),
        local_config_path: PathBuf::new(),
        local_plaintext_config: false,
        metadata_filename_key: std::sync::Arc::default(),
    };
    let state = crate::dashboard_event_tests::fixture_state();
    *state.vault.write().await = Some(vault);
    let response = crate::export_owner_age_recipient(axum::extract::State(state))
        .await
        .unwrap();
    assert_eq!(
        response.headers()[axum::http::header::CACHE_CONTROL],
        "private, no-store"
    );
    let body = axum::body::to_bytes(response.into_body(), 4096)
        .await
        .unwrap();
    assert_eq!(body.as_ref(), user_recipient.as_bytes());
    assert_ne!(body.as_ref(), group_recipient.as_bytes());
    assert!(!String::from_utf8(body.to_vec()).unwrap().contains("SECRET"));
}

#[test]
fn owner_recipient_derives_only_existing_user_identity_without_side_effects() {
    let directory =
        std::env::temp_dir().join(format!("rawrarr-owner-recipient-{}", uuid::Uuid::new_v4()));
    fs::create_dir(&directory).unwrap();
    let path = directory.join("age-identity");
    let device = directory.join("node-age-identity");
    let group = directory.join("group-age-identity");
    fs::write(&device, "not a user identity").unwrap();
    fs::write(&group, "not a user identity").unwrap();
    assert_eq!(remembered_user_recipient_at(&path).unwrap(), None);
    assert!(!path.exists());
    let (identity, expected) = AgeVault::generate_identity().unwrap();
    let identity = Zeroizing::new(identity);
    fs::write(&path, identity.as_bytes()).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
    assert_eq!(remembered_user_recipient_at(&path).unwrap(), Some(expected));
    assert_eq!(fs::read(&path).unwrap(), identity.as_bytes());
    assert_eq!(fs::read_to_string(&device).unwrap(), "not a user identity");
    assert_eq!(fs::read_to_string(&group).unwrap(), "not a user identity");
    fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();
    assert!(remembered_user_recipient_at(&path).is_err());
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
    fs::write(&path, "invalid existing user identity").unwrap();
    assert!(remembered_user_recipient_at(&path).is_err());
    fs::remove_file(&path).unwrap();
    std::os::unix::fs::symlink(&device, &path).unwrap();
    assert!(remembered_user_recipient_at(&path).is_err());
    fs::remove_dir_all(&directory).unwrap();
}
