# Bootstrap the group-encrypted shared config

## Objective

Make the first approved metadata node create `/shared/config.aged` encrypted to the group age recipient and keep the coordinator Configuration page clear when no valid node route is available. Issue [#146](https://github.com/willjasen/rawrarr/issues/146).

## Initial state

The node initialized and distributed its group age identity but did not write `config.aged` until a user later saved settings. The browser Configuration page passed `route.base` to the loader without validating it, allowing a null value to cause the observed `reading 'base'` exception.

## Changes and verification

- First age-vault initialization now writes a default empty shared config through the existing atomic writer, which encrypts with the group's recipient. Existing config files are left intact.
- Added route validation in the coordinator page and shared-config loader.
- `node --test scripts/tests/test_rawrarr_coordinator_workspace.mjs` passed (27 checks); `git diff --check` passed.
- The host Rust test could not start because the host Rust toolchain aborts while loading LLVM: its configured Z3 dylib is missing. The isolated OrbStack runner passed Node 110/110, Playwright browser 1/1, and Rust 87/87 (including the group-recipient encryption assertions). Python integration checks were skipped by that runner. `git diff --check` passed.
- Internal read-only inspection found the MacBook fossil has an existing Oct 22 `/shared/config.aged` but no group identity/envelope artifacts; the running image is labeled revision `38b4d482`. The Mac mini fossil's separate named `/shared` volume has no `config.aged` or group identity files.
- Both Macs run native Syncthing, but neither active Syncthing configuration contains a Rawrarr folder. The node volumes are separate named Docker volumes, so the existing config cannot reach the Mac mini through the current setup. No peer/folder configuration or runtime state was changed.
- PR [#148](https://github.com/willjasen/rawrarr/pull/148) merged to GitHub `main` as `83d70be134a1a56e81a109ce62a75ad13632d17c`. The source image was built from that published revision on both target Macs.
- Deployed only `roarlay` in `rawrarr-internal-mini` and `fossil` in `rawrarr-node-internal`; the separate Mac mini `rawrarr-internal` fossil container was left untouched. The coordinator image is `sha256:20c8786c7de3d278d94cfe541cbe97056addcbf1dbccffc67e7cfc65428f1f24`; the MacBook node image was labeled with the same source revision. PostgreSQL and Tailscale were not recreated.
- Before deployment, saved a mode-600 PostgreSQL dump and mode-600 fossil `/data` and `/shared` archives in protected host backup directories. The PostgreSQL dump passed `gzip -t`; both volume archives passed `gzip -t`. Previous images remain tagged `rawrarr:rollback-roarlay-83d70be-before` and `rawrarr:rollback-fossil-83d70be-before`.
- Both target services report healthy with zero restarts. PostgreSQL is accepting connections. Coordinator HTTPS `/health`, MacBook HTTPS `/health`, and `/direct-api/health` returned HTTP 200. Tailscale Serve read-back confirms coordinator HTTPS `/` still proxies to `http://127.0.0.1:8080`. Both per-container deployment locks were removed after verification.
- The MacBook's existing `config.aged` remained present and unchanged; it still has no group-recipient artifacts until its owner age identity is unlocked against the updated node. The Mac mini fossil's separate `/shared` volume remains without `config.aged`. Although native Syncthing runs on both Macs, neither Syncthing configuration contains a Rawrarr folder. At the user's direction, Syncthing pairing is on standby, so file replication and first-unlock migration are not yet verified.

## Recovery

Revert the task-owned changes to `rust-arr/src/secure_config.rs`, `rust-arr/src/coordinator-workspace.mjs`, `rust-arr/web-export/metadata-export.js`, the generated `rust-arr/src/assets/metadata-export.js`, the coordinator workspace test, this runbook, and the index entry. Restore only the affected container from its recorded rollback image if needed; PostgreSQL and fossil volume backups are retained separately. The initialization only creates a missing ciphertext; it does not alter existing config data.
