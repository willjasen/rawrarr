use super::{EnrollmentRequest, ciphertext_export_path, encrypted_session_request, valid_node_id, validate_origin};

#[test]
fn coordinator_relay_accepts_only_ciphertext_export_reads() {
    assert!(ciphertext_export_path("api/metadata/export/mini"));
    assert!(ciphertext_export_path("api/metadata/export/config"));
    assert!(ciphertext_export_path(&format!("api/metadata/export/ciphertext/text/{}.json.aged", "a".repeat(64))));
    assert!(!ciphertext_export_path(""));
    assert!(!ciphertext_export_path("api/radarr/movies"));
    assert!(!ciphertext_export_path("api/metadata/movies/search"));
    assert!(!ciphertext_export_path("api/metadata/export/ciphertext/text/../../secret"));
    assert!(ciphertext_export_path("api/metadata/export/ciphertext/artwork/123.aged"));
}

#[test]
fn coordinator_relay_accepts_only_age_wrapped_session_key_posts_for_encrypted_library() {
    let method = axum::http::Method::POST;
    let public = b"-----BEGIN AGE ENCRYPTED FILE-----\nwrapped-session-key";
    assert!(encrypted_session_request("api/metadata/session/library", &method, public.as_ref()));
    assert!(!encrypted_session_request("api/radarr/movies", &method, public.as_ref()));
    assert!(!encrypted_session_request("api/metadata/session/library", &axum::http::Method::GET, public.as_ref()));
    assert!(!encrypted_session_request("api/metadata/session/library", &method, &[4; 65]));
    assert!(!encrypted_session_request("api/metadata/session/library", &method, public[..10].as_ref()));
}

#[test]
fn node_unwraps_age_wrapped_session_key_for_browser_response() {
    use std::{io::Write, process::{Command, Stdio}};
    let Ok(keygen) = Command::new("age-keygen").arg("-o").arg("/tmp/rawrarr-session-test-identity").output() else { return; };
    assert!(keygen.status.success());
    let identity = std::fs::read_to_string("/tmp/rawrarr-session-test-identity").expect("generated identity");
    let _ = std::fs::remove_file("/tmp/rawrarr-session-test-identity");
    let recipient = Command::new("age-keygen").args(["-y", "-"]).stdin(Stdio::piped()).stdout(Stdio::piped()).spawn();
    let Ok(mut recipient) = recipient else { return; };
    recipient.stdin.take().expect("stdin").write_all(identity.as_bytes()).expect("identity input");
    let output = recipient.wait_with_output().expect("recipient output");
    assert!(output.status.success());
    let recipient = String::from_utf8(output.stdout).expect("recipient UTF-8");
    let session_key = [0x5au8; 32];
    let mut encrypt = Command::new("age").args(["--recipient", recipient.trim()]).stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().expect("age encrypt");
    encrypt.stdin.take().expect("stdin").write_all(&session_key).expect("key input");
    let ciphertext = encrypt.wait_with_output().expect("ciphertext");
    assert!(ciphertext.status.success());
    let decrypted = crate::secure_config::decrypt_age_input(identity.as_bytes(), &ciphertext.stdout)
        .expect("node decrypts wrapped session key");
    assert!(decrypted.status.success());
    assert_eq!(decrypted.stdout, session_key);
}

#[test]
fn enrollment_request_contains_no_private_identity_or_shared_data_fields() {
    let valid = serde_json::json!({
        "nodeId":"mac-mini-storage", "role":"storage", "displayName":"Mac mini",
        "origin":"http://localhost:7297", "rpId":"localhost"
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
