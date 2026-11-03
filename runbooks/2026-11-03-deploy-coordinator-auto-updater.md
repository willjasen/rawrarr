# Deploy the do3 coordinator auto-updater

Issue: [#349](https://github.com/willjasen/rawrarr/issues/349)

Pull requests: [#350](https://github.com/willjasen/rawrarr/pull/350), [#353](https://github.com/willjasen/rawrarr/pull/353), [#354](https://github.com/willjasen/rawrarr/pull/354)

## Installation

- Installed from authenticated GitHub `main` revision `0162d3dfa74ff1ef23f4a4a92fee9e55c5cb8ab2` on do3. The archive SHA-256 was `0c582888ef68624b4ba9cefe263987a90ff04de82f2409732f96d95f4bdb59f4` and was verified on do3 before extraction.
- Installed `/opt/rawrarr-coordinator-auto-updater/auto_update_coordinators.py` and the systemd service/timer with `deploy/do3/install-coordinator-auto-updater.sh`.
- Enabled `rawrarr-coordinator-auto-update.timer`. It polls every 60 seconds using the existing `/root/gh-token` Contents-read credential; the token is not printed or copied elsewhere.
- Ran `systemd-analyze verify` on both units and started one immediate poll. The unit completed successfully and skipped `0162d3d` because the changes since the deployed coordinator revision were docs, tests, and deployment helper files.
- The poller consumes only published `main`, skips docs/tests/workflows/landing-page-only changes, coalesces rapid commits, builds native amd64 once with BuildKit cache, deploys internal first, and promotes the same immutable image to public/live. It waits on the existing per-service locks, preserves rollback image tags, and recreates only `roarlay` with no dependency changes and no pulls.
- The main poller token cannot read Actions check runs or commit status. The service relies on the repository's normal review and merge process for changes entering `main`; it never builds branch refs or local worktrees.

## Current deployment state

- Both internal and public/live do3 coordinators are healthy on published revision `e10250a8ef8ff694b41c44609c2a4962d7d3c030`, image ID `sha256:c70678674ff0117b06914210fa47be5ec96a3ee23f421d943e422204d84ea952`, `linux/amd64`.
- Both `/api/version` responses report that revision and deployment time `2026-11-03T05:21:10Z`; the signed-in coordinator workspace displays the timestamp in its sidebar.
- Internal local/HTTPS health and the Tailscale Serve route passed verification.
- Public container health plus its in-container `/health` and `/api/version` passed. Cloudflare returned HTTP 403 for `https://roarlay.rawrarr.net/health` from do3, so the public edge route remains unverified from that host. The auto-updater logs a warning for a non-200 edge response after local service health succeeds and leaves ingress policy unchanged.
- The internal/public locks were released. Existing databases, Tailscale state, UI volumes, secrets, and unrelated services remain intact.

## Compatibility and operations

No persistent data, database schema, or protocol format changed, so no migration is needed. The timer installs deployment-time metadata through host-local Compose overrides; each coordinator keeps its own timestamp. To inspect the poller, use `systemctl status rawrarr-coordinator-auto-update.timer` and `journalctl -u rawrarr-coordinator-auto-update.service`. To update the poller implementation or its systemd units, rerun the installer from a verified published `main` archive.

## Verification

- `python3 -m unittest discover -s scripts/tests -p 'test_*.py' -v` — 50 tests passed.
- `python3 scripts/tests/test_do3_coordinator_auto_updater.py` — 6 tests passed.
- `node --test scripts/tests/test_rawrarr_coordinator_build_info.mjs` — 2 tests passed.
- `python3 -m py_compile deploy/do3/auto_update_coordinators.py`, `bash -n deploy/do3/install-coordinator-auto-updater.sh`, and `git diff --check` — passed.
- GitHub PR #354 `Basic script tests` check — passed.
- `systemd-analyze verify` — both installed units passed.
- Immediate systemd service run — succeeded and correctly skipped the current non-runtime-only main changes.
