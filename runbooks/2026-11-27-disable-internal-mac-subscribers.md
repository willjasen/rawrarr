# Disable internal Mac subscriber nodes

Issue: [#477](https://github.com/willjasen/rawrarr/issues/477)

## Objective and initial state

Disable the internal Rawrarr subscriber applications on Brandon's MacBook Pro
and Mac mini while preserving their `/data`, `/shared`, identities,
credentials, Tailscale, and unrelated services. The user intends to use public
production. Project guidance identifies the Macs as `fossil` subscribers in
Compose project `rawrarr-node-internal`; the internal coordinator is separate
on do3.

The repository was clean on `main` at `ab535001383513d1f032f59fb91bbdfd146017f5`.
`origin/main` was fetched and matched that commit. Task worktree:
`/Users/willjasen/AppData/GitHub/rawrarr-worktrees/disable-internal-mac-subscribers`
on `codex/disable-internal-mac-subscribers`.

## Actions and findings

- Created issue #477 with acceptance criteria for stopping only the two
  subscriber application services and retaining their data.
- Confirmed `https://rawrarr.net/` responds HTTP 200 as the public landing
  page. The live public app health route `https://roarlay.rawrarr.net/health`
  responds HTTP 200. The apex is not the app hostname.
- Tried the documented SSH aliases to both Macs. MacBook SSH stopped at
  strict host-key verification (`Host key verification failed`). Mac mini
  SSH reported the configured identity file missing and then denied
  authentication. Tailscale SSH to both hosts stopped because no ED25519 host
  key is known. No host identity checks were bypassed.
- The local machine identifies as `Brandons-MacBook-Pro-1622.local`, but its
  Docker context is `desktop-linux`; the Docker daemon socket is unavailable.
  No subscriber inventory, deployment locks, or volumes could be inspected.
- No Mac, coordinator, or production service was changed.

## Verification and remaining work

The public URL checks above succeeded. Mac node shutdown is **not completed**:
verified SSH and Docker access are required to check each container lock and
stop only its `fossil` application service. Preserve Tailscale and all volumes;
verify stopped status and production availability after access is restored.

Resolve the MacBook and Mac mini host-key trust through a verified channel and
restore the Mac mini's configured SSH identity (or a documented authenticated
Tailscale SSH path). Then continue this issue and runbook. Do not accept an
unknown host key or disable strict checking.
