# Authenticated WebRTC workspace routing

Issue: [#495](https://github.com/willjasen/rawrarr/issues/495). Related
Rayfish enrollment: [#489](https://github.com/willjasen/rawrarr/issues/489).

## Objective and initial state

Add a bounded WebRTC data path from the authenticated Rawrarr workspace to an
approved subscriber node, retaining the coordinator route as fallback. Keep
Rayfish mesh enrollment separate; #489 may use this channel for protected invite
delivery. Do not deploy or change live services.

The task uses branch `codex/webrtc-workspace-routing` in its own worktree from
fresh `origin/main` at `c09633aa`. Existing direct Tailscale routing uses a
short-lived node-scoped session; the coordinator proxy has an explicit API
allowlist. No WebRTC endpoint, signaling, or browser transport exists yet.

## Implementation notes

Added a pinned `webrtc-rs` 0.21.1 dependency and a test-only local offer/answer
data-channel probe. The probe creates two native peers bound to loopback UDP,
exchanges complete ICE descriptions directly, and asserts a message round trip;
it uses no signaling server, STUN or TURN. The crate requires Rust 1.91 because
its pinned `rtc-mdns` dependency uses `Ipv4Addr::from_octets`, stabilized in
Rust 1.91. The runtime and reusable test image builders were both updated.

This is transport qualification groundwork only. Authenticated signaling,
node identity proof, bounded workspace operations, browser routing and fallback
remain unimplemented. The test image rebuild is in progress.

## Verification

- `git diff --check` — passed.
- Prior run on Rust 1.88 failed while compiling `rtc-mdns 0.21.1`:
  `Ipv4Addr::from_octets` is unstable on that compiler.
- The reusable Rust 1.91 test image is rebuilding; the focused data-channel
  test has not yet completed.

## Adoption and rollback

No database or persistent state migration is expected if sessions and signaling
remain ephemeral. Mixed-version clients and nodes must retain coordinator
fallback. Rollback disables the opt-in WebRTC route and leaves existing direct
Tailscale and coordinator routes unchanged. Do not deploy before issue #495's
security review and qualification checks pass.
