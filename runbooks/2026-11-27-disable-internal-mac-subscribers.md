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
- On resumption, used the user's local `~/.ssh/id_ed25519` with strict
  host-key checking. Both Mac host keys matched saved `known_hosts` entries.
  No SSH trust settings or key files were changed.
- MacBook Pro uses Docker context `orbstack` and reports `linux/x86_64`. Its
  `rawrarr-node-internal` project had healthy `fossil`, `artwork-cache`, and
  `tailscale` containers. The fossil had Docker volumes mounted at `/data` and
  `/shared`.
- Mac mini uses Docker context `orbstack` and reports `linux/aarch64`. Its
  `rawrarr-internal` project had healthy `fossil`, `config-cache`, and
  `tailscale` containers. The fossil had Docker volumes mounted at `/data`,
  `/shared`, and `/ui`.
- No `.container-rebuild-fossil.lock` markers were present on either Mac.
  Claimed each marker atomically, then ran `docker compose stop fossil` in each
  host's existing Compose project with its exact active Compose files and
  image override. Both commands completed. Each fossil now reports state
  `exited`; the markers were removed after verification.
- The `artwork-cache` and `tailscale` services remain running on the MacBook;
  the `config-cache` and `tailscale` services remain running on the Mac mini.
  No volumes, credentials, identities, Tailscale configuration, coordinator,
  or public production services were changed.

## Verification and final state

- Both `rawrarr-node-internal-fossil-1` (MacBook Pro) and
  `rawrarr-internal-fossil-1` (Mac mini) report `exited` after the stop.
- Confirmed both per-container deployment markers are absent after work.
- Confirmed the unrelated services listed above remain running.
- After both stops, `https://rawrarr.risk-mermaid.ts.net/health` and
  `https://roarlay.rawrarr.net/health` each returned HTTP 200.
- The live public app is `https://roarlay.rawrarr.net/`; `https://rawrarr.net/`
  is the public landing page.

No persistent data was changed, so no migration or data rollback is needed.
To re-enable a subscriber later, use its existing Compose project and
configuration and verify its intended coordinator endpoint before starting
`fossil`. Do not use `down -v` or remove existing volumes.
