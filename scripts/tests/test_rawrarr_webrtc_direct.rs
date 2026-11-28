use super::{
    MAX_WORKSPACE_FRAME_BYTES, WORKSPACE_PROTOCOL_VERSION, WorkspaceMethod, WorkspaceRequest,
    WorkspaceResponse,
};
use std::sync::Arc;
use std::time::Duration;

use bytes::BytesMut;
use rtc::ice::mdns::MulticastDnsMode;
use tokio::sync::Notify;
use webrtc::data_channel::{DataChannel, DataChannelEvent};
use webrtc::peer_connection::{
    MediaEngine, PeerConnection, PeerConnectionBuilder, PeerConnectionEventHandler,
    RTCConfigurationBuilder, RTCIceGatheringState, Registry, SettingEngineBuilder,
    register_default_interceptors,
};
use webrtc::runtime::{Runtime, default_runtime};

fn workspace_request(path: &str, method: WorkspaceMethod) -> WorkspaceRequest {
    WorkspaceRequest {
        version: WORKSPACE_PROTOCOL_VERSION,
        request_id: uuid::Uuid::new_v4(),
        session_token: "a".repeat(64),
        method,
        path: path.to_owned(),
        body: None,
    }
}

#[test]
fn workspace_request_frames_are_versioned_bounded_and_allowlisted() {
    let request = workspace_request("/api/radarr/live-library", WorkspaceMethod::Get);
    let frame = request.encode().expect("allowed operation encodes");
    assert!(frame.len() <= MAX_WORKSPACE_FRAME_BYTES);
    let decoded = WorkspaceRequest::decode(&frame).expect("valid request decodes");
    assert_eq!(decoded.path, request.path);
    assert_eq!(decoded.request_id, request.request_id);
    assert!(WorkspaceRequest::decode(b"{\"version\":2}").is_err());

    let arbitrary = workspace_request("/api/anything", WorkspaceMethod::Get);
    assert!(arbitrary.encode().is_err());
    let traversal = workspace_request("/api/radarr/../settings", WorkspaceMethod::Get);
    assert!(traversal.encode().is_err());
    let oversized = workspace_request("/api/prowlarr/search", WorkspaceMethod::Post);
    assert!(oversized.encode().is_ok());
    let mut oversized = oversized;
    oversized.body = Some(serde_json::json!({"query": "x".repeat(MAX_WORKSPACE_FRAME_BYTES)}));
    assert!(oversized.encode().is_err());
}

#[test]
fn workspace_response_frames_reject_oversized_and_invalid_statuses() {
    let response = WorkspaceResponse {
        version: WORKSPACE_PROTOCOL_VERSION,
        request_id: uuid::Uuid::new_v4(),
        status: 200,
        body: Some(serde_json::json!({"ok": true})),
    };
    assert!(response.encode().is_ok());
    let oversized = WorkspaceResponse {
        body: Some(serde_json::json!({"data": "x".repeat(MAX_WORKSPACE_FRAME_BYTES)})),
        ..response
    };
    assert!(oversized.encode().is_err());
}

#[derive(Clone)]
struct GatherHandler(Arc<Notify>);

#[async_trait::async_trait]
impl PeerConnectionEventHandler for GatherHandler {
    async fn on_ice_gathering_state_change(&self, state: RTCIceGatheringState) {
        if state == RTCIceGatheringState::Complete {
            self.0.notify_one();
        }
    }
}

#[derive(Clone)]
struct EchoHandler {
    runtime: Arc<dyn Runtime>,
    gathered: Arc<Notify>,
}

#[async_trait::async_trait]
impl PeerConnectionEventHandler for EchoHandler {
    async fn on_ice_gathering_state_change(&self, state: RTCIceGatheringState) {
        if state == RTCIceGatheringState::Complete {
            self.gathered.notify_one();
        }
    }

    async fn on_data_channel(&self, channel: Arc<dyn DataChannel>) {
        self.runtime.spawn(Box::pin(async move {
            while let Some(event) = channel.poll().await {
                match event {
                    DataChannelEvent::OnMessage(message) => {
                        let _ = channel.send(BytesMut::from(message.data.as_ref())).await;
                    }
                    DataChannelEvent::OnClose => break,
                    _ => {}
                }
            }
        }));
    }
}

async fn local_peer(
    runtime: Arc<dyn Runtime>,
    handler: Arc<dyn PeerConnectionEventHandler>,
) -> Arc<dyn PeerConnection> {
    let mut media = MediaEngine::default();
    media
        .register_default_codecs()
        .expect("default codecs register");
    let interceptors = register_default_interceptors(Registry::new(), &mut media)
        .expect("default interceptors register");
    Arc::new(
        PeerConnectionBuilder::new()
            .with_configuration(RTCConfigurationBuilder::new().build())
            .with_media_engine(media)
            .with_interceptor_registry(interceptors)
            .with_setting_engine(
                SettingEngineBuilder::new()
                    .with_multicast_dns_mode(MulticastDnsMode::Disabled)
                    .build(),
            )
            .with_handler(handler)
            .with_runtime(runtime)
            .with_udp_addrs(vec!["127.0.0.1:0"])
            .with_data_channel_send_buffer_limit(64 * 1024)
            .build()
            .await
            .expect("peer connection builds"),
    )
}

async fn wait_for_ice(notify: &Notify) {
    tokio::time::timeout(Duration::from_secs(10), notify.notified())
        .await
        .expect("ICE gathering completes");
}

#[tokio::test]
async fn local_data_channel_round_trips_without_signaling_or_turn_services() {
    let runtime = default_runtime().expect("Tokio WebRTC runtime is enabled");
    let offer_gathered = Arc::new(Notify::new());
    let answer_gathered = Arc::new(Notify::new());
    let offer_peer = local_peer(
        runtime.clone(),
        Arc::new(GatherHandler(offer_gathered.clone())),
    )
    .await;
    let answer_peer = local_peer(
        runtime.clone(),
        Arc::new(EchoHandler {
            runtime,
            gathered: answer_gathered.clone(),
        }),
    )
    .await;

    let channel = offer_peer
        .create_data_channel("rawrarr-workspace-v1", None)
        .await
        .expect("data channel creates");
    let offer = offer_peer.create_offer(None).await.expect("offer creates");
    offer_peer
        .set_local_description(offer)
        .await
        .expect("offer becomes local description");
    wait_for_ice(&offer_gathered).await;
    let offer = offer_peer
        .local_description()
        .await
        .expect("local offer exists");

    answer_peer
        .set_remote_description(offer)
        .await
        .expect("remote offer is accepted");
    let answer = answer_peer
        .create_answer(None)
        .await
        .expect("answer creates");
    answer_peer
        .set_local_description(answer)
        .await
        .expect("answer becomes local description");
    wait_for_ice(&answer_gathered).await;
    let answer = answer_peer
        .local_description()
        .await
        .expect("local answer exists");
    offer_peer
        .set_remote_description(answer)
        .await
        .expect("remote answer is accepted");

    let round_trip = tokio::time::timeout(Duration::from_secs(10), async {
        let mut opened = false;
        while let Some(event) = channel.poll().await {
            match event {
                DataChannelEvent::OnOpen => {
                    opened = true;
                    channel.send_text("rawrarr-local-probe").await.unwrap();
                }
                DataChannelEvent::OnMessage(message) if opened => {
                    return message.data.to_vec();
                }
                DataChannelEvent::OnClose => break,
                _ => {}
            }
        }
        Vec::new()
    })
    .await
    .expect("data channel opens and exchanges a message");

    assert_eq!(round_trip, b"rawrarr-local-probe");
    offer_peer.close().await.expect("offer connection closes");
    answer_peer.close().await.expect("answer connection closes");
}

#[test]
fn node_proof_binds_challenge_session_node_and_both_descriptions() {
    let session = uuid::Uuid::new_v4();
    let challenge = "a".repeat(64);
    let payload =
        super::node_proof_payload(session, "node-a", &challenge, "offer", "answer").unwrap();
    for changed in [
        super::node_proof_payload(
            uuid::Uuid::new_v4(),
            "node-a",
            &challenge,
            "offer",
            "answer",
        )
        .unwrap(),
        super::node_proof_payload(session, "node-b", &challenge, "offer", "answer").unwrap(),
        super::node_proof_payload(session, "node-a", &"b".repeat(64), "offer", "answer").unwrap(),
        super::node_proof_payload(session, "node-a", &challenge, "other", "answer").unwrap(),
        super::node_proof_payload(session, "node-a", &challenge, "offer", "other").unwrap(),
    ] {
        assert_ne!(payload, changed);
    }
    assert!(super::node_proof_payload(session, "node-a", "short", "offer", "answer").is_err());
    assert!(
        super::node_proof_payload(session, "node-a", &challenge, &"x".repeat(49153), "answer")
            .is_err()
    );
}

#[test]
fn node_proof_requires_the_pinned_root_approved_node_and_exact_transcript() {
    use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
    use ml_dsa::{
        MlDsa65, SigningKey,
        signature::{Keypair, Signer},
    };

    let root_key = SigningKey::<MlDsa65>::from_seed(&[91u8; 32].into());
    let node_key = SigningKey::<MlDsa65>::from_seed(&[92u8; 32].into());
    let root_public = root_key.verifying_key().encode().to_vec();
    let node_public = node_key.verifying_key().encode().to_vec();
    let root_fingerprint = crate::device_signing::fingerprint(&root_public);
    let node_key_hex = node_public
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let signed_payload = serde_json::json!({
        "domain": "rawrarr-device-approval-v1",
        "nodeId": "node-a",
        "role": "metadata",
        "publicKeyHex": node_key_hex,
    });
    let certificate_signature = root_key
        .sign(&crate::device_signing::canonical_json(&signed_payload).unwrap())
        .encode();
    let certificate = serde_json::json!({
        "algorithm": crate::device_signing::ALGORITHM,
        "rootPublicKey": BASE64.encode(&root_public),
        "rootFingerprint": root_fingerprint,
        "signingPublicKeyHex": node_key_hex,
        "signingFingerprint": crate::device_signing::node_fingerprint(&node_public),
        "nodeId": "node-a",
        "role": "metadata",
        "signedPayload": signed_payload,
        "signature": BASE64.encode(certificate_signature),
    });

    let session = uuid::Uuid::new_v4();
    let challenge = "a".repeat(64);
    let proof = super::sign_node_proof(
        session,
        "node-a",
        &challenge,
        "offer-sdp",
        "answer-sdp",
        |message| Ok(node_key.sign(message).encode().to_vec()),
    )
    .unwrap();
    assert!(super::verify_node_proof(
        &certificate,
        &root_fingerprint,
        &proof,
        session,
        "node-a",
        &challenge,
        "offer-sdp",
        "answer-sdp",
    ));
    assert!(!super::verify_node_proof(
        &certificate,
        &"0".repeat(64),
        &proof,
        session,
        "node-a",
        &challenge,
        "offer-sdp",
        "answer-sdp",
    ));
    assert!(!super::verify_node_proof(
        &certificate,
        &root_fingerprint,
        &proof,
        session,
        "node-b",
        &challenge,
        "offer-sdp",
        "answer-sdp",
    ));
    assert!(!super::verify_node_proof(
        &certificate,
        &root_fingerprint,
        &proof,
        session,
        "node-a",
        &challenge,
        "altered-offer",
        "answer-sdp",
    ));
    let mut altered = proof;
    altered["signature"] = serde_json::Value::String("00".to_owned());
    assert!(!super::verify_node_proof(
        &certificate,
        &root_fingerprint,
        &altered,
        session,
        "node-a",
        &challenge,
        "offer-sdp",
        "answer-sdp",
    ));
}
