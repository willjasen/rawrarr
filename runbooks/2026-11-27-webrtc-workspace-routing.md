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

Pending investigation and implementation.

## Verification

Pending.

## Adoption and rollback

No database or persistent state migration is expected if sessions and signaling
remain ephemeral. Mixed-version clients and nodes must retain coordinator
fallback. Rollback disables the opt-in WebRTC route and leaves existing direct
Tailscale and coordinator routes unchanged. Do not deploy before issue #495's
security review and qualification checks pass.
