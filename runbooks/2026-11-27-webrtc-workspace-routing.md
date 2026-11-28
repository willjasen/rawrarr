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

This increment adds bounded, in-memory coordinator signaling endpoints for
authenticated workspace offers, approved-node offer pickup, node answer
publication, and owner-only answer retrieval. Signaling expires after 45
seconds, is one-time claimed by the target node, is capped at 48 KiB per SDP,
and is never persisted. Total outstanding sessions and per-owner sessions are
bounded. The node's signed coordinator client methods are present, but no node
worker calls them yet. Therefore node identity proof, data-channel request
dispatch, browser routing, and fallback remain unimplemented. The signaling
handlers and helper tests compile and pass, but end-to-end HTTP authorization,
deployed node pickup, and multi-instance coordinator behavior have not been
qualified.

## Verification

- `git diff --check` — passed.
- Prior run on Rust 1.88 failed while compiling `rtc-mdns 0.21.1`:
  `Ipv4Addr::from_octets` is unstable on that compiler.
- The reusable AMD64 test image built with Rust 1.91.1.
- `cargo test --locked --offline --manifest-path /workspace/rust-arr/Cargo.toml webrtc_direct::tests -- --nocapture` — passed (3/3) in a disposable native clone. The bridge was disconnected after dependency preparation; a unique project-labeled internal network kept loopback ICE available without an external route.
- `cargo test --locked --offline --manifest-path /workspace/rust-arr/Cargo.toml webrtc_ -- --nocapture` — passed (5/5) after adding coordinator signaling, in a new disposable linked clone with its default bridge disconnected and a unique internal network.
- `cargo test --locked --offline --manifest-path /workspace/rust-arr/Cargo.toml subscriber_payloads_cannot_be_proxied_to_coordinator -- --nocapture` — passed (1/1) in the same clone.
- Repeated both focused Rust commands after rebasing onto current `origin/main` (including Syncthing XML hardening); results remained 5/5 and 1/1.
- Build warning remains: `NodeAuth::poll_webrtc_offer` and `publish_webrtc_answer` are unused until the node worker is implemented.

## Adoption and rollback

Coordinator signaling stores ephemeral offers and answers in process memory;
it changes no database schema, node-local state, or synchronized files. The
WebRTC transport dependencies remain dev/test-only and the runtime image/toolchain
is unchanged. Existing nodes ignore the new routes, so no adopter migration is
needed for this increment. Restarting a coordinator drops pending sessions;
clients must time out and retry through the existing coordinator route. A
multi-replica coordinator deployment would require sticky routing or shared
ephemeral signaling before enabling this feature. Once request dispatch ships,
it must retain coordinator fallback for mixed-version nodes and clients.
Rollback removes the new routes and in-memory state while keeping current
direct Tailscale and coordinator paths available. Do not deploy before issue
#495's security review and qualification checks pass.

## Continued signaling review

Offer pickup now checks expiry at the claim boundary itself, rather than relying
on its caller to prune first. Added a regression case for an expired offer with
no cleanup pass. This closes a stale-offer acceptance path for future polling
callers. Node peer worker and browser integration remain pending. No persistent
state or migration impact. Verification: diff whitespace check passed; the new
Rust regression has not yet been executed.
