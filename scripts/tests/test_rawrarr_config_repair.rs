use super::*;
use ml_dsa::{MlDsa65, SigningKey, signature::{Keypair, Signer}};
use axum::{Router, extract::{Request, State}, routing::get};

fn revision(counter: u64, device: &str, digest: &str) -> Revision {
    Revision { schema_version: 1, root_fingerprint: "a".repeat(64), device_id: device.into(), counter,
        operation: "put".into(), ciphertext_digest: Some(digest.into()), certificate: serde_json::Value::Null, signature: format!("sig-{device}-{counter}") }
}

#[test]
fn accepts_only_bounded_age_ciphertext() {
    assert!(valid_ciphertext(b"age-encryption.org/v1\nabc"));
    assert!(!valid_ciphertext(b"{\"radarrApiKey\":\"secret\"}"));
    assert!(!valid_ciphertext(b"age-encryption.org/v1\n"));
    assert!(!valid_ciphertext(&[AGE_MAGIC, &vec![b'x'; MAX_CONFIG_BYTES]].concat()));
}

#[test]
fn peer_allowlist_requires_stable_ids_and_direct_tailnet_https_urls() {
    assert!(valid_peer_id("device-123abc"));
    assert!(!valid_peer_id("../device"));
    assert!(valid_peer_url("https://rawrarr.risk-mermaid.ts.net/direct-api"));
    assert!(!valid_peer_url("http://rawrarr.risk-mermaid.ts.net/direct-api"));
    assert!(!valid_peer_url("https://example.com/direct-api"));
    assert!(!valid_peer_url("https://user@rawrarr.risk-mermaid.ts.net/direct-api"));
    assert!(!valid_peer_url("https://rawrarr.risk-mermaid.ts.net/direct-api?token=bad"));
}

#[test]
fn configured_merkle_peer_count_excludes_the_local_node() {
    let peer = PeerRpc::fixture("node-a", &"a".repeat(64), HashMap::from([
        ("node-a".to_owned(), "https://node-a.example.ts.net/direct-api".to_owned()),
        ("node-b".to_owned(), "https://node-b.example.ts.net/direct-api".to_owned()),
    ]));
    assert_eq!(peer.configured_remote_peer_count(), 1);
}

#[test]
fn peer_status_exposes_successfully_reached_tailnet_peers() {
    let status=PeerSyncStatus::default();
    status.update_for_test("root",0,1,vec!["node-b".into()]);
    assert_eq!(status.snapshot()["reachablePeers"],serde_json::json!(["node-b"]));
}

#[test]
fn repaired_config_is_written_as_exact_private_ciphertext() {
    let root=std::env::temp_dir().join(format!("rawrarr-config-repair-{}",uuid::Uuid::new_v4().simple()));
    let path=root.join("shared").join("config.aged");
    let ciphertext=b"age-encryption.org/v1\nopaque-ciphertext";
    write_active_ciphertext(&path,ciphertext).unwrap();
    assert_eq!(std::fs::read(&path).unwrap(),ciphertext);
    #[cfg(unix)] {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,0o600);
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[cfg(unix)]
#[test]
fn config_ciphertext_reads_reject_symlinks() {
    use std::os::unix::fs::symlink;

    let root=std::env::temp_dir().join(format!("rawrarr-config-symlink-{}",uuid::Uuid::new_v4().simple()));
    std::fs::create_dir_all(&root).unwrap();
    let target=root.join("ciphertext");
    let path=root.join("config.aged");
    std::fs::write(&target,b"age-encryption.org/v1\nopaque-ciphertext").unwrap();
    symlink(&target,&path).unwrap();
    assert!(read_ciphertext_file(&path).is_err());
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn merkle_is_stable_and_conflict_order_ignores_wall_clock() {
    let records = vec![revision(5, "node-b", "b"), revision(5, "node-a", "c"), revision(4, "node-z", "z")];
    assert_eq!(merkle_root(&records), merkle_root(&records));
    assert_eq!(merkle_root(&records), merkle_root(&records.iter().cloned().rev().collect::<Vec<_>>()));
    assert_eq!(winner(&records).unwrap().device_id, "node-b");
    assert_ne!(merkle_root(&records), merkle_root(&records[..2]));
    assert_eq!(merkle_root(&[]), merkle_root(&[]));
}

#[test]
fn peer_challenges_are_bound_to_allowlisted_cert_method_path_and_one_use() {
    let root_key=SigningKey::<MlDsa65>::from_seed(&[21u8;32].into());
    let node_key=SigningKey::<MlDsa65>::from_seed(&[22u8;32].into());
    let root_public=root_key.verifying_key().encode().to_vec();
    let node_public=node_key.verifying_key().encode().to_vec();
    let root_fingerprint=crate::device_signing::fingerprint(&root_public);
    let node_hex=encode_hex(&node_public);
    let signed_payload=serde_json::json!({"domain":"rawrarr-device-approval-v1","nodeId":"node-a","role":"metadata","publicKeyHex":node_hex});
    let payload_bytes=crate::device_signing::canonical_json(&signed_payload).unwrap();
    let root_signature=root_key.sign(&payload_bytes).encode();
    let certificate=serde_json::json!({
        "algorithm":"ML-DSA-65","rootPublicKey":BASE64.encode(root_public),"rootFingerprint":root_fingerprint,"nodeId":"node-a","role":"metadata",
        "signingPublicKeyHex":node_hex,"signingFingerprint":crate::device_signing::node_fingerprint(&node_public),
        "signedPayload":signed_payload,"signature":BASE64.encode(root_signature)
    });
    assert!(crate::device_signing::verify_certificate(&certificate,&root_fingerprint));
    let mut revision=Revision {schema_version:1,root_fingerprint:root_fingerprint.clone(),device_id:"node-a".into(),counter:1,operation:"put".into(),ciphertext_digest:Some("b".repeat(64)),certificate:certificate.clone(),signature:String::new()};
    let revision_payload=crate::device_signing::canonical_json(&revision.payload()).unwrap();
    revision.signature=encode_hex(node_key.sign(&revision_payload).encode().as_ref());
    assert!(revision.verify(&root_fingerprint));
    let mut tampered=revision.clone();tampered.counter+=1;assert!(!tampered.verify(&root_fingerprint));
    let receiver=PeerRpc::fixture("node-b",&root_fingerprint,HashMap::from([("node-a".to_owned(),"https://node-a".to_owned())]));
    let (nonce,_)=receiver.issue_challenge().unwrap();
    let request_payload=serde_json::json!({"domain":"rawrarr-config-peer-request-v1","version":1,"senderId":"node-a","receiverId":"node-b","method":"GET","path":"/config-peer/manifest","bodySha256":hex(&Sha256::digest([])),"nonce":nonce});
    let request_bytes=crate::device_signing::canonical_json(&request_payload).unwrap();
    let request_signature=node_key.sign(&request_bytes).encode();
    let mut headers=HeaderMap::new();
    headers.insert("x-rawrarr-peer-id","node-a".parse().unwrap());
    headers.insert("x-rawrarr-peer-certificate",BASE64.encode(serde_json::to_vec(&certificate).unwrap()).parse().unwrap());
    headers.insert("x-rawrarr-peer-nonce",nonce.parse().unwrap());
    headers.insert("x-rawrarr-peer-signature",encode_hex(request_signature.as_ref()).parse().unwrap());
    assert!(crate::device_signing::verify(&node_public,request_signature.as_ref(),&request_bytes));
    let cert_text=headers.get("x-rawrarr-peer-certificate").unwrap().to_str().unwrap();
    let parsed:serde_json::Value=serde_json::from_slice(&BASE64.decode(cert_text).unwrap()).unwrap();
    let public=decode_hex(parsed.get("signingPublicKeyHex").unwrap().as_str().unwrap()).unwrap();
    let signature=decode_hex(headers.get("x-rawrarr-peer-signature").unwrap().to_str().unwrap()).unwrap();
    let nonce_header=headers.get("x-rawrarr-peer-nonce").unwrap().to_str().unwrap();
    let payload=serde_json::json!({"domain":"rawrarr-config-peer-request-v1","version":1,"senderId":"node-a","receiverId":"node-b","method":"GET","path":"/config-peer/manifest","bodySha256":hex(&Sha256::digest([])),"nonce":nonce_header});
    assert!(crate::device_signing::verify(&public,&signature,&crate::device_signing::canonical_json(&payload).unwrap()));
    assert!(receiver.authorize(&headers,"GET","/config-peer/manifest"));
    assert!(!receiver.authorize(&headers,"GET","/config-peer/manifest"));
}

#[tokio::test]
async fn two_peer_rpc_reconcile_exchanges_a_signed_manifest_over_the_config_route() {
    let root_key=SigningKey::<MlDsa65>::from_seed(&[31u8;32].into());
    let node_key=SigningKey::<MlDsa65>::from_seed(&[32u8;32].into());
    let root_public=root_key.verifying_key().encode().to_vec();
    let node_public=node_key.verifying_key().encode().to_vec();
    let root_fingerprint=crate::device_signing::fingerprint(&root_public);
    let node_hex=encode_hex(&node_public);
    let signed_payload=serde_json::json!({"domain":"rawrarr-device-approval-v1","nodeId":"node-a","role":"metadata","publicKeyHex":node_hex});
    let approval=root_key.sign(&crate::device_signing::canonical_json(&signed_payload).unwrap()).encode();
    let certificate=serde_json::json!({"algorithm":"ML-DSA-65","rootPublicKey":BASE64.encode(root_public),"rootFingerprint":root_fingerprint,
        "nodeId":"node-a","role":"metadata","signingPublicKeyHex":node_hex,"signingFingerprint":crate::device_signing::node_fingerprint(&node_public),
        "signedPayload":signed_payload,"signature":BASE64.encode(approval)});

    async fn challenge(State(peer):State<PeerRpc>)->Json<serde_json::Value>{
        let (nonce,receiver_id)=peer.issue_challenge().unwrap();
        Json(serde_json::json!({"version":1,"nonce":nonce,"receiverId":receiver_id}))
    }
    async fn manifest(State(peer):State<PeerRpc>,request:Request)->Result<Json<serde_json::Value>,StatusCode>{
        if !peer.authorize(request.headers(),"GET",request.uri().path()){return Err(StatusCode::UNAUTHORIZED);}
        Ok(Json(serde_json::json!({"schemaVersion":1,"merkleRoot":merkle_root(&[]),"total":0,"offset":0,"revisions":[]})))
    }
    let receiver=PeerRpc::fixture("node-b",&root_fingerprint,HashMap::from([("node-a".into(),"http://unused".into())]));
    let app=Router::new().route("/config-peer/challenge",get(challenge)).route("/config-peer/manifest",get(manifest)).with_state(receiver);
    let listener=tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address=listener.local_addr().unwrap();
    tokio::spawn(async move { let _=axum::serve(listener,app).await; });
    let sender=PeerRpc::fixture("node-a",&root_fingerprint,HashMap::from([("node-b".into(),format!("http://{address}"))]));
    let mirror=CiphertextMirror{client:None,root_fingerprint:root_fingerprint.clone()};
    assert!(!sender.reconcile_once(&mirror,Path::new("/path/which/does/not/exist"),Some(&certificate),&PeerSyncStatus::default()).await);
}
