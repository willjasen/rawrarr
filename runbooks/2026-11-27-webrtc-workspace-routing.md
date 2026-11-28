# Authenticated WebRTC workspace routing

Issue: [#495](https://github.com/willjasen/rawrarr/issues/495). Related
Rayfish enrollment: [#489](https://github.com/willjasen/rawrarr/issues/489).

## Objective and initial state

Add a bounded WebRTC data path from the authenticated Rawrarr workspace to an
approved subscriber node, retaining the coordinator route as fallback. Keep
Rayfish mesh enrollment separate; #489 may use this channel for protected invite
delivery. Do not deploy or change live services.

Branch `codex/webrtc-workspace-routing` has its own worktree. It was created
from `origin/main` at `c09633aa`, then rebased onto
`06007b36` after six newer main commits landed. Existing direct Tailscale routing
uses a short-lived node-scoped session; the coordinator proxy has an explicit API
allowlist. No WebRTC endpoint, signaling, or browser transport exists yet.

## Implementation notes

Added pinned `webrtc-rs` 0.21.1 dev dependencies and a local offer/answer
data-channel probe. The probe creates two native peers bound to loopback UDP,
exchanges complete ICE descriptions directly, and asserts a message round trip;
it uses no signaling server, STUN or TURN. The test also defines versioned JSON
request/response frames capped at 48 KiB, validates a 64-hex-character node
session token, and rejects operations outside a fixed workspace API allowlist.
The `rtc-mdns` dependency uses `Ipv4Addr::from_octets`, stabilized in Rust 1.91,
so only the reusable test image baseline was updated. The production package
and runtime builder remain on Rust 1.88 while WebRTC is test-only.

This is transport and protocol groundwork only. Authenticated signaling, node
identity proof, data-channel request dispatch, browser routing and fallback
remain unimplemented.

## Verification

- `git diff --check` — passed.
- Prior run on Rust 1.88 failed while compiling `rtc-mdns 0.21.1`:
  `Ipv4Addr::from_octets` is unstable on that compiler.
- The reusable AMD64 test image built with Rust 1.91.1.
- `cargo test --locked --offline --manifest-path /workspace/rust-arr/Cargo.toml webrtc_direct::tests -- --nocapture` — passed (3/3) in a disposable native clone. The bridge was disconnected after dependency preparation; a unique project-labeled internal network kept loopback ICE available without an external route.

## Adoption and rollback

This increment changes no runtime, database, persistent state or coordinator/node
protocol. The WebRTC dependency and request schema are dev/test-only; no adopter
migration is needed yet. Once request dispatch ships, it must retain coordinator
fallback for mixed-version nodes and clients. Rollback removes the opt-in route
while keeping current direct Tailscale and coordinator paths available. Do not
deploy before issue #495's security review and qualification checks pass.
