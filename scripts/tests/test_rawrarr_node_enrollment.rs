use super::{EnrollmentRequest, ciphertext_export_path, encrypted_session_request, valid_node_id, validate_origin};

#[test]
fn coordinator_relay_accepts_only_ciphertext_export_reads() {
    assert!(ciphertext_export_path("api/metadata/export/mini"));
    assert!(ciphertext_export_path("api/metadata/export/config"));
    assert!(ciphertext_export_path("api/metadata/export/node-recipient"));
    assert!(ciphertext_export_path(&format!("api/metadata/export/ciphertext/text/{}.json.aged", "a".repeat(64))));
    assert!(!ciphertext_export_path(""));
    assert!(!ciphertext_export_path("api/radarr/movies"));
    assert!(!ciphertext_export_path("api/metadata/movies/search"));
    assert!(!ciphertext_export_path("api/metadata/export/ciphertext/text/../../secret"));
    assert!(ciphertext_export_path("api/metadata/export/ciphertext/artwork/123.aged"));
}

#[test]
fn coordinator_relay_accepts_only_valid_ephemeral_key_posts_for_encrypted_library() {
    let method = axum::http::Method::POST;
    let key = p256::SecretKey::random(&mut p256::elliptic_curve::rand_core::OsRng);
    let public = key.public_key().to_sec1_bytes();
    assert!(encrypted_session_request("api/metadata/session/library", &method, public.as_ref()));
    assert!(!encrypted_session_request("api/radarr/movies", &method, public.as_ref()));
    assert!(!encrypted_session_request("api/metadata/session/library", &axum::http::Method::GET, public.as_ref()));
    assert!(!encrypted_session_request("api/metadata/session/library", &method, &[4; 65]));
    assert!(!encrypted_session_request("api/metadata/session/library", &method, public[..64].as_ref()));
}

#[test]
fn enrollment_request_contains_no_private_identity_or_shared_data_fields() {
    let valid = serde_json::json!({
        "nodeId":"mac-mini-storage", "role":"storage", "displayName":"Mac mini",
        "origin":"http://localhost:7297", "rpId":"localhost",
        "signingPublicKeyHex":"00".repeat(1952), "keyProofSignatureHex":"00".repeat(3309)
    });
    assert!(serde_json::from_value::<EnrollmentRequest>(valid.clone()).is_ok());
    let mut with_private_key = valid;
    with_private_key["ageIdentity"] = serde_json::json!("AGE-SECRET-KEY-EXAMPLE");
    assert!(serde_json::from_value::<EnrollmentRequest>(with_private_key).is_err());
    let mut with_user = serde_json::json!({
        "nodeId":"mac-mini-storage", "role":"storage", "displayName":"Mac mini",
        "origin":"http://localhost:7297", "rpId":"localhost", "userId":"attacker-chosen"
    });
    assert!(serde_json::from_value::<EnrollmentRequest>(with_user.take()).is_err());
    assert!(validate_origin("localhost", "http://localhost:7297").is_ok());
}

#[test]
fn enrollment_sender_serializes_only_approved_public_metadata() {
    let sender = include_str!("../../rust-arr/src/node_auth.rs");
    assert!(sender.contains("nodeId\":node_id"));
    assert!(sender.contains("role\":role"));
    assert!(sender.contains("displayName\":display_name"));
    assert!(sender.contains("origin\":origin"));
    assert!(sender.contains("rpId\":rp_id"));
    assert!(sender.contains("signingPublicKeyHex"));
    assert!(sender.contains("keyProofSignatureHex"));
    assert!(!sender.contains("ageIdentity"));
    assert!(!sender.contains("age_identity"));
}

#[test]
fn enrollment_accepts_only_safe_ids_and_browser_origins() {
    for id in ["mac-mini-storage", "metadata_01"] {
        assert!(valid_node_id(id));
    }
    for id in ["", "../node", "node name", "x".repeat(129).as_str()] {
        assert!(!valid_node_id(id));
    }
    for origin in [
        "http://example.com",
        "https://user@example.com",
        "https://example.com/path",
        "https://evil.invalid",
    ] {
        assert!(validate_origin("example.com", origin).is_err());
    }
    assert!(validate_origin("example.com", "https://rawrarr.example.com").is_ok());
}

#[test]
fn enrollment_allows_same_credential_and_role_to_upgrade_an_existing_node_id() {
    let coordinator = include_str!("../../rust-arr/src/coordinator.rs");
    assert!(coordinator.contains("token_sha256 IS DISTINCT FROM $2 OR role <> $3"));
}
