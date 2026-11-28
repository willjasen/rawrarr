use super::*;

#[test]
fn rayfish_peer_route_requires_opt_in_and_dedicated_http_loopback_port() {
    for url in ["http://127.0.0.1:17443", "http://127.0.0.1:17999/"] {
        assert!(rayfish_peer_url(url));
        let mut peer = PeerRpc::fixture("node-a", &"a".repeat(64), HashMap::new());
        assert!(peer.client_for_peer(url).is_err(), "HTTP routes must be disabled by default");
        peer.rayfish_http = true;
        assert!(peer.client_for_peer(url).is_ok());
    }
    for url in [
        "https://127.0.0.1:17443",
        "http://127.0.0.2:17443",
        "http://localhost:17443",
        "http://192.0.2.1:17443",
        "http://127.0.0.1:80",
        "http://127.0.0.1:18000",
        "http://127.0.0.1:17443/direct-api",
        "http://127.0.0.1:17443/?q=1",
        "http://user@127.0.0.1:17443",
    ] {
        assert!(!rayfish_peer_url(url), "{url}");
    }
    assert!(valid_peer_url("https://fossil.example.ts.net/direct-api"));
    assert!(!valid_peer_url("http://127.0.0.1:17443"));
}
use ml_dsa::{MlDsa65, SigningKey, signature::{Keypair, Signer}};
use axum::{Router, body::Body, extract::{Path as AxumPath, Request, State}, http::header, response::Response, routing::get};
use std::{collections::HashMap, process::Command, sync::Arc};

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

fn fossil_revision(path: &str, counter: u64, device: &str, operation: &str, digest: Option<&str>) -> FossilRevision {
    FossilRevision { schema_version: 1, root_fingerprint: "a".repeat(64), object_path: path.into(), device_id: device.into(), counter,
        operation: operation.into(), ciphertext_digest: digest.map(str::to_owned), certificate: serde_json::Value::Null, signature: format!("sig-{device}-{counter}") }
}

#[test]
fn fossil_object_paths_are_fixed_to_aged_metadata_under_the_domain() {
    for path in ["metadata/movies/text/a.json.aged", "metadata/tv/artwork/poster.aged", "metadata/library/text/hash.json.aged"] {
        assert!(valid_fossil_path(path), "{path}");
    }
    for path in ["config.aged", "../metadata/movies/text/a.aged", "metadata/../../secret.aged", "metadata/movies/a.json", "metadata\\movies\\a.aged", "metadata/movies//a.aged"] {
        assert!(!valid_fossil_path(path), "{path}");
    }
}

#[test]
fn fossil_merkle_and_conflict_order_are_deterministic_and_keep_tombstones() {
    let older = fossil_revision("metadata/movies/text/a.json.aged", 2, "node-a", "put", Some(&"b".repeat(64)));
    let tombstone = fossil_revision("metadata/movies/text/a.json.aged", 3, "node-b", "delete", None);
    assert_eq!(fossil_merkle_root(&[older.clone(), tombstone.clone()]), fossil_merkle_root(&[tombstone.clone(), older.clone()]));
    assert_ne!(fossil_merkle_root(&[older.clone()]), fossil_merkle_root(&[older.clone(), tombstone.clone()]));
    let winner = fossil_winners(&[older, tombstone]).remove("metadata/movies/text/a.json.aged").unwrap();
    assert_eq!(winner.operation, "delete");
}

#[test]
fn app_config_revision_binds_app_epoch_grant_fence_and_tombstone() {
    let root_key = SigningKey::<MlDsa65>::from_seed(&[91u8; 32].into());
    let device_key = SigningKey::<MlDsa65>::from_seed(&[92u8; 32].into());
    let root_public = root_key.verifying_key().encode().to_vec();
    let device_public = device_key.verifying_key().encode().to_vec();
    let root_fingerprint = crate::device_signing::fingerprint(&root_public);
    let app_id = "11111111-1111-4111-8111-111111111111";
    let grant_payload = serde_json::json!({
        "domain":"rawrarr-app-key-grant-v1","schemaVersion":1,"appInstanceId":app_id,
        "appType":"radarr","appRecipient":"age1pq1app","deviceId":"node-a",
        "deviceRecipient":"age1pq1device","keyEpoch":4,"encryptedIdentitySha256":"a".repeat(64)
    });
    let grant_signature = root_key.sign(&crate::device_signing::canonical_json(&grant_payload).unwrap()).encode();
    let proof = serde_json::json!({"algorithm":crate::device_signing::ALGORITHM,"rootPublicKey":BASE64.encode(&root_public),
        "rootFingerprint":root_fingerprint,"signedPayload":grant_payload,"signature":BASE64.encode(grant_signature)});
    let grant_proof = BASE64.encode(serde_json::to_vec(&proof).unwrap());
    let fence_payload = serde_json::json!({"domain":"rawrarr-app-writer-fence-v1","schemaVersion":1,"appInstanceId":app_id,
        "keyEpoch":4,"writerDeviceId":"node-a","fencingGeneration":7});
    let fence_signature = root_key.sign(&crate::device_signing::canonical_json(&fence_payload).unwrap()).encode();
    let fence = serde_json::json!({"rootPublicKey":BASE64.encode(&root_public),"rootFingerprint":root_fingerprint,
        "signedPayload":fence_payload,"signature":BASE64.encode(fence_signature)});
    let writer_fence = BASE64.encode(serde_json::to_vec(&fence).unwrap());
    let signed_device_approval = serde_json::json!({"domain":"rawrarr-device-approval-v1","nodeId":"node-a","role":"metadata","publicKeyHex":encode_hex(&device_public)});
    let device_approval_signature = root_key.sign(&crate::device_signing::canonical_json(&signed_device_approval).unwrap()).encode();
    let certificate = serde_json::json!({"algorithm":"ML-DSA-65","rootPublicKey":BASE64.encode(&root_public),"rootFingerprint":root_fingerprint,
        "nodeId":"node-a","role":"metadata","signingPublicKeyHex":encode_hex(&device_public),
        "signingFingerprint":crate::device_signing::node_fingerprint(&device_public),"signedPayload":signed_device_approval,
        "signature":BASE64.encode(device_approval_signature)});
    let mut revision = AppConfigRevision { schema_version:1, root_fingerprint:root_fingerprint.clone(), app_instance_id:app_id.into(),
        key_epoch:4, device_id:"node-a".into(), fencing_generation:7,
        operation_id:"22222222-2222-4222-8222-222222222222".into(), operation:"delete".into(), ciphertext_digest:None,
        grant_proof, writer_fence, certificate, signature:String::new() };
    revision.signature = encode_hex(device_key.sign(&crate::device_signing::canonical_json(&revision.payload()).unwrap()).encode().as_ref());
    assert!(revision.verify(&root_fingerprint));
    assert_ne!(app_config_merkle_root(&[]), app_config_merkle_root(&[revision.clone()]));
    let mut cross_app = revision.clone();
    cross_app.app_instance_id = "33333333-3333-4333-8333-333333333333".into();
    assert!(!cross_app.verify(&root_fingerprint), "owner grant is bound to one app instance");
    let mut stale_epoch = revision.clone();
    stale_epoch.key_epoch = 3;
    assert!(!stale_epoch.verify(&root_fingerprint), "owner grant is bound to one key epoch");
    let mut stale_fence = revision.clone();
    stale_fence.fencing_generation = 8;
    stale_fence.signature = encode_hex(device_key.sign(&crate::device_signing::canonical_json(&stale_fence.payload()).unwrap()).encode().as_ref());
    assert!(!stale_fence.verify(&root_fingerprint), "a node cannot advance the owner-issued fencing generation");
    revision.operation = "put".into();
    assert!(!revision.verify(&root_fingerprint), "put revisions require a ciphertext digest");
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
    std::fs::create_dir_all(root.join("shared")).unwrap();
    let shared=std::fs::canonicalize(root.join("shared")).unwrap();
    let path=shared.join("config.aged");
    let ciphertext=b"age-encryption.org/v1\nopaque-ciphertext";
    write_active_ciphertext(&path,ciphertext, &shared).unwrap();
    assert_eq!(std::fs::read(&path).unwrap(),ciphertext);
    #[cfg(unix)] {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,0o600);
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn config_repair_rejects_paths_outside_the_configured_shared_root() {
    let root=std::env::temp_dir().join(format!("rawrarr-config-repair-root-{}",uuid::Uuid::new_v4().simple()));
    let outside=std::env::temp_dir().join(format!("rawrarr-config-repair-outside-{}",uuid::Uuid::new_v4().simple()));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::create_dir_all(&outside).unwrap();
    let error=write_active_ciphertext(&outside.join("config.aged"), b"age-encryption.org/v1\nopaque", &root).unwrap_err();
    assert_eq!(error.kind(),std::io::ErrorKind::PermissionDenied);
    assert!(!outside.join("config.aged").exists());
    std::fs::remove_dir_all(root).unwrap();
    std::fs::remove_dir_all(outside).unwrap();
}

#[cfg(unix)]
#[test]
fn config_repair_rejects_symlinked_parent_components() {
    use std::os::unix::fs::symlink;
    let root=std::env::temp_dir().join(format!("rawrarr-config-repair-link-{}",uuid::Uuid::new_v4().simple()));
    let outside=std::env::temp_dir().join(format!("rawrarr-config-repair-target-{}",uuid::Uuid::new_v4().simple()));
    std::fs::create_dir_all(&root).unwrap(); std::fs::create_dir_all(&outside).unwrap();
    symlink(&outside,root.join("link")).unwrap();
    let error=write_active_ciphertext(&root.join("link").join("config.aged"), b"age-encryption.org/v1\nopaque", &root).unwrap_err();
    assert_eq!(error.kind(),std::io::ErrorKind::PermissionDenied);
    assert!(!outside.join("config.aged").exists());
    std::fs::remove_dir_all(root).unwrap(); std::fs::remove_dir_all(outside).unwrap();
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
    let mut fossil=FossilRevision {schema_version:1,root_fingerprint:root_fingerprint.clone(),object_path:"metadata/movies/text/42.json.aged".into(),device_id:"node-a".into(),counter:1,operation:"put".into(),ciphertext_digest:Some("c".repeat(64)),certificate:certificate.clone(),signature:String::new()};
    let fossil_payload=crate::device_signing::canonical_json(&fossil.payload()).unwrap();
    fossil.signature=encode_hex(node_key.sign(&fossil_payload).encode().as_ref());
    assert!(fossil.verify(&root_fingerprint));
    let mut tampered_fossil=fossil.clone();tampered_fossil.object_path="metadata/../secret.aged".into();assert!(!tampered_fossil.verify(&root_fingerprint));
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

#[tokio::test]
async fn two_node_fossil_rpc_reconnects_resolves_conflicts_and_applies_tombstones() {
    const CHILD: &str = "RAWARR_FOSSIL_RPC_TEST_CHILD";
    if std::env::var(CHILD).as_deref() != Ok("1") {
        let root_key=SigningKey::<MlDsa65>::from_seed(&[71u8;32].into());
        let root_fingerprint=crate::device_signing::fingerprint(&root_key.verifying_key().encode());
        let scratch=std::env::temp_dir().join(format!("rawrarr-fossil-rpc-{}",uuid::Uuid::new_v4().simple()));
        std::fs::create_dir_all(&scratch).unwrap();
        for node_id in ["node-a","node-b"] {
            let node_root=scratch.join(node_id);
            let output=Command::new(std::env::current_exe().unwrap())
                .args(["--exact","config_repair::tests::two_node_fossil_rpc_reconnects_resolves_conflicts_and_applies_tombstones","--nocapture"])
                .env(CHILD,"1")
                .env("RAWARR_FOSSIL_RPC_TEST_NODE",node_id)
                .env("RAWARR_INSTANCE_ROOT_FINGERPRINT",&root_fingerprint)
                .env("RAWARR_LOCAL_DIR",node_root.join("local"))
                .env("RAWARR_EVENT_DIR",node_root.join("evt"))
                .env("RAWARR_EVT_WRITE_MODE","evt")
                .output().unwrap();
            assert!(output.status.success(),"isolated {node_id} RPC test failed:\n{}{}",String::from_utf8_lossy(&output.stdout),String::from_utf8_lossy(&output.stderr));
        }
        std::fs::remove_dir_all(scratch).unwrap();
        return;
    }

    let node_id=std::env::var("RAWARR_FOSSIL_RPC_TEST_NODE").unwrap();
    let peer_id=if node_id=="node-a" {"node-b"} else {"node-a"};
    let root_fingerprint=std::env::var("RAWARR_INSTANCE_ROOT_FINGERPRINT").unwrap();
    let event=std::env::var("RAWARR_EVENT_DIR").unwrap();
    crate::secure_config::AgeVault::ensure_node_signing_key().unwrap();

    fn certificate(root_key:&SigningKey<MlDsa65>, node_id:&str, node_public:&[u8])->serde_json::Value {
        let public_hex=encode_hex(node_public);
        let signed_payload=serde_json::json!({"domain":"rawrarr-device-approval-v1","nodeId":node_id,"role":"metadata","publicKeyHex":public_hex});
        let approval=root_key.sign(&crate::device_signing::canonical_json(&signed_payload).unwrap()).encode();
        serde_json::json!({"algorithm":"ML-DSA-65","rootPublicKey":BASE64.encode(root_key.verifying_key().encode()),"rootFingerprint":crate::device_signing::fingerprint(&root_key.verifying_key().encode()),
            "nodeId":node_id,"role":"metadata","signingPublicKeyHex":public_hex,"signingFingerprint":crate::device_signing::node_fingerprint(node_public),
            "signedPayload":signed_payload,"signature":BASE64.encode(approval)})
    }
    fn make_revision(path:&str,device:&str,counter:u64,operation:&str,bytes:Option<&[u8]>,cert:&serde_json::Value,sign:impl FnOnce(&[u8])->Vec<u8>)->FossilRevision {
        let digest=bytes.map(|data|hex(&Sha256::digest(data)));
        let mut revision=FossilRevision {schema_version:1,root_fingerprint:std::env::var("RAWARR_INSTANCE_ROOT_FINGERPRINT").unwrap(),object_path:path.into(),device_id:device.into(),counter,operation:operation.into(),ciphertext_digest:digest,certificate:cert.clone(),signature:String::new()};
        revision.signature=encode_hex(&sign(&crate::device_signing::canonical_json(&revision.payload()).unwrap()));
        revision
    }

    let root_key=SigningKey::<MlDsa65>::from_seed(&[71u8;32].into());
    let local_public=crate::secure_config::AgeVault::node_signing_public_key().unwrap();
    let local_certificate=certificate(&root_key,&node_id,&local_public);
    let peer_seed=if peer_id=="node-a" {[81u8;32]} else {[82u8;32]};
    let peer_key=SigningKey::<MlDsa65>::from_seed(&peer_seed.into());
    let peer_public=peer_key.verifying_key().encode().to_vec();
    let peer_certificate=certificate(&root_key,peer_id,&peer_public);

    let conflict_path="metadata/movies/text/conflict.json.aged";
    let reconnect_path="metadata/movies/text/reconnect.json.aged";
    let tombstone_path="metadata/movies/text/deleted.json.aged";
    let local_conflict:&[u8]=b"age-encryption.org/v1\nlocal-conflict";
    let peer_conflict:&[u8]=b"age-encryption.org/v1\npeer-conflict";
    let local_old:&[u8]=b"age-encryption.org/v1\nold-offline-copy";
    let peer_new:&[u8]=b"age-encryption.org/v1\nreconnected-copy";
    let local_deleted:&[u8]=b"age-encryption.org/v1\nwill-be-deleted";
    let fossil_root=std::path::PathBuf::from(&event).join("fossil");
    for (path,bytes,counter) in [(conflict_path,local_conflict,4),(reconnect_path,local_old,1),(tombstone_path,local_deleted,2)] {
        let full=fossil_root.join(path);std::fs::create_dir_all(full.parent().unwrap()).unwrap();std::fs::write(&full,bytes).unwrap();
        cache_fossil_object(bytes).unwrap();
        let revision=make_revision(path,&node_id,counter,"put",Some(bytes),&local_certificate,|payload|crate::secure_config::AgeVault::sign_node_message(payload).unwrap());
        import_fossil_revision(&revision).unwrap();
    }

    let peer_revisions=vec![
        make_revision(conflict_path,peer_id,4,"put",Some(peer_conflict),&peer_certificate,|payload|{let encoded=peer_key.sign(payload).encode();encoded.to_vec()}),
        make_revision(reconnect_path,peer_id,5,"put",Some(peer_new),&peer_certificate,|payload|{let encoded=peer_key.sign(payload).encode();encoded.to_vec()}),
        make_revision(tombstone_path,peer_id,7,"delete",None,&peer_certificate,|payload|{let encoded=peer_key.sign(payload).encode();encoded.to_vec()}),
    ];
    let objects=HashMap::from([
        (hex(&Sha256::digest(peer_conflict)),peer_conflict.to_vec()),
        (hex(&Sha256::digest(peer_new)),peer_new.to_vec()),
    ]);
    #[derive(Clone)] struct MockPeer { auth:PeerRpc, revisions:Arc<Vec<FossilRevision>>, objects:Arc<HashMap<String,Vec<u8>>> }
    async fn challenge(State(peer):State<MockPeer>)->Json<serde_json::Value> {
        let (nonce,receiver_id)=peer.auth.issue_challenge().unwrap();
        Json(serde_json::json!({"version":1,"nonce":nonce,"receiverId":receiver_id}))
    }
    async fn manifest(State(peer):State<MockPeer>,request:Request)->Result<Json<serde_json::Value>,StatusCode> {
        if !peer.auth.authorize(request.headers(),"GET",request.uri().path()) {return Err(StatusCode::UNAUTHORIZED);}
        Ok(Json(serde_json::json!({"schemaVersion":1,"merkleRoot":fossil_merkle_root(&peer.revisions),"total":peer.revisions.len(),"offset":0,"revisions":peer.revisions.as_ref()})))
    }
    async fn object(State(peer):State<MockPeer>,AxumPath(digest):AxumPath<String>,request:Request)->Result<Response<Body>,StatusCode> {
        if !peer.auth.authorize(request.headers(),"GET",request.uri().path()) {return Err(StatusCode::UNAUTHORIZED);}
        let bytes=peer.objects.get(&digest).ok_or(StatusCode::NOT_FOUND)?.clone();
        Response::builder().header(header::CONTENT_TYPE,"application/octet-stream").body(Body::from(bytes)).map_err(|_|StatusCode::INTERNAL_SERVER_ERROR)
    }
    let listener=tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address=listener.local_addr().unwrap();
    let auth=PeerRpc::fixture(peer_id,&root_fingerprint,HashMap::from([(node_id.clone(),format!("http://{address}"))]));
    let mock=MockPeer {auth,revisions:Arc::new(peer_revisions),objects:Arc::new(objects)};
    let app=Router::new().route("/config-peer/challenge",get(challenge)).route("/config-peer/fossil-manifest",get(manifest)).route("/config-peer/fossil-object/{digest}",get(object)).with_state(mock);
    tokio::spawn(async move {let _=axum::serve(listener,app).await;});

    let mut rpc=PeerRpc::fixture(&node_id,&root_fingerprint,HashMap::from([(peer_id.to_owned(),format!("http://{address}"))]));
    rpc.rayfish_http=true;
    let repaired=rpc.reconcile_fossil_once(Some(&local_certificate)).await;
    assert!(repaired,"peer reconnect should repair missing/stale ciphertexts and apply the tombstone");
    let expected_conflict=if peer_id=="node-b" {peer_conflict} else {local_conflict};
    assert_eq!(std::fs::read(fossil_root.join(conflict_path)).unwrap(),expected_conflict,"equal counters resolve lexicographically by stable device ID");
    assert_eq!(std::fs::read(fossil_root.join(reconnect_path)).unwrap(),peer_new,"stale offline object must receive the newer peer ciphertext");
    assert!(!fossil_root.join(tombstone_path).exists(),"a signed peer tombstone must remove a prior local object");
    assert!(load_fossil_revisions().iter().any(|revision|revision.device_id==peer_id&&revision.object_path==reconnect_path));
    assert!(!rpc.reconcile_fossil_once(Some(&local_certificate)).await,"a second reconciliation must be idempotent");
}
