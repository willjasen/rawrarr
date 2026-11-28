use super::{
    WEBRTC_SDP_LIMIT, WEBRTC_SIGNAL_TTL, WebRtcSignal, claim_webrtc_offer, prune_webrtc_signals,
    valid_webrtc_sdp,
};
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};
use uuid::Uuid;

const SDP: &str = "v=0\r\nm=application 9 UDP/DTLS/SCTP webrtc-datachannel\r\na=ice-ufrag:abcd\r\na=ice-pwd:abcdefgh\r\n";

#[test]
fn signaling_sdp_is_strictly_validated_and_bounded() {
    assert!(valid_webrtc_sdp(SDP));
    assert!(!valid_webrtc_sdp(
        "v=0\r\na=ice-ufrag:abcd\r\na=ice-pwd:abcdefgh\r\n"
    ));
    assert!(!valid_webrtc_sdp(
        "v=0\nm=application 9 UDP/DTLS/SCTP webrtc-datachannel\na=ice-ufrag:abcd\na=ice-pwd:abcdefgh\n"
    ));
    assert!(!valid_webrtc_sdp(&format!(
        "{SDP}{}",
        "x".repeat(WEBRTC_SDP_LIMIT)
    )));
}

#[test]
fn signaling_offers_are_node_scoped_one_time_and_expire() {
    let mut signals = HashMap::new();
    let first_id = Uuid::new_v4();
    let second_id = Uuid::new_v4();
    let old_id = Uuid::new_v4();
    let owner_id = Uuid::new_v4();
    for (id, node_id, created_at) in [
        (first_id, "metadata-a", Instant::now()),
        (second_id, "metadata-a", Instant::now()),
        (
            old_id,
            "metadata-b",
            Instant::now() - WEBRTC_SIGNAL_TTL - Duration::from_secs(1),
        ),
    ] {
        signals.insert(
            id,
            WebRtcSignal {
                owner_user_id: owner_id,
                node_id: node_id.to_owned(),
                offer_sdp: SDP.to_owned(),
                answer_sdp: None,
                created_at,
                claimed: false,
            },
        );
    }

    assert!(claim_webrtc_offer(&mut signals, "unknown-node").is_none());
    let first =
        claim_webrtc_offer(&mut signals, "metadata-a").expect("first owner offer is delivered");
    assert!(matches!(first.session_id, id if id == first_id || id == second_id));
    let second =
        claim_webrtc_offer(&mut signals, "metadata-a").expect("second offer is delivered once");
    assert_ne!(first.session_id, second.session_id);
    assert!(claim_webrtc_offer(&mut signals, "metadata-a").is_none());
    prune_webrtc_signals(&mut signals);
    assert!(!signals.contains_key(&old_id));
}
