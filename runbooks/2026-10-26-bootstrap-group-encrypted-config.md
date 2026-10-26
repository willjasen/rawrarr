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
- Internal rollout remains pending until the code is published to `main`; receiving the MacBook ciphertext on the Mac mini also requires an approved Syncthing shared-folder relationship, which is absent in the current host configurations.

## Recovery

Revert the task-owned changes to `rust-arr/src/secure_config.rs`, `rust-arr/src/coordinator-workspace.mjs`, `rust-arr/web-export/metadata-export.js`, the generated `rust-arr/src/assets/metadata-export.js`, the coordinator workspace test, this runbook, and the index entry. The initialization only creates a missing ciphertext; it does not alter existing config data.
