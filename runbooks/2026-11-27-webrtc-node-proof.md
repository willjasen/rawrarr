# WebRTC node transcript proof

Issue: [#495](https://github.com/willjasen/rawrarr/issues/495)

## Objective and initial state

Implement the next safe #495 increment: sign a node proof that binds the
signaling session, selected node, browser challenge, and both SDP descriptions;
verify it against the selected node's instance-root-approved signing
certificate. The merged branch had only a proof-payload builder. The WebRTC
stack remains test-only, and no production peer worker or browser channel is
enabled.

Created branch `codex/webrtc-node-proof` in its own worktree from fetched
`origin/main` at `3c32716`. No deployment or persistent-state change is in
scope.

## Implementation

Added a signer callback that canonicalizes the domain-separated transcript and
returns a compact proof envelope with a hex signature. The callback boundary
keeps private signing material in the existing node-local signer. Added proof
verification that first checks the certificate against the expected root
fingerprint, then checks the selected node ID and exact transcript before
verifying the node signature. The verifier rejects altered challenges,
sessions, node IDs, either SDP description, malformed signatures, and an
untrusted root.

Added deterministic tests using disposable ML-DSA root and node keys. Updated
the test index to distinguish primitive proof verification from the still
unimplemented browser transmission and peer worker.

## Verification

- `git diff --check` — passed.
- `rustfmt --edition 2024 --check rust-arr/src/webrtc_direct.rs scripts/tests/test_rawrarr_webrtc_direct.rs` — passed.
- In a unique `rawrarr-test-regression` linked clone prepared from the current worktree and disconnected from the default bridge:
  `cargo test --locked --offline --manifest-path /workspace/rust-arr/Cargo.toml webrtc_ -- --nocapture` — passed (8/8). This includes signaling, loopback channel, frame bounds, and proof checks.
- Compile emitted the known unused `NodeAuth::poll_webrtc_offer` and `publish_webrtc_answer` warning. No new warnings were reported.

## Adoption, limits, and rollback

This is additive code with no schema, protocol runtime, node state, or
coordinator behavior change. Existing deployments require no migration. The
proof functions are not wired to a WebRTC worker and do not establish that a
browser or node transmits or verifies a proof in a live session. No deployment
was performed. Rollback is removal of the new proof helpers, their focused
test, and the test-index wording; no persistent state needs recovery.

## Final state

The focused signing and verification primitives are implemented and pass the
documented offline Rust checks. The node worker, browser proof exchange,
request dispatch, and fallback remain open work under #495.
