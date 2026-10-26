# Encrypt shared metadata to the group identity

Issue [#109](https://github.com/willjasen/rawrarr/issues/109).

## Objective
Make group age identity a runtime encryption identity: nodes encrypt shared metadata and config to the group recipient, each node installs a group-key envelope encrypted to its device age recipient, and the browser decrypts a user-wrapped group-key envelope locally.

## Initial state
Started from fetched `origin/main` at `8524cc0` in dedicated worktree `/Users/willjasen/.codex/worktrees/issue-109-group-age-metadata/rawrarr`, branch `codex/issue-109-group-age-metadata`. Current `AgeVault` uses the user age identity as both config decryption identity and shared-file recipient. Storage scans read the public `age-recipient` file and encrypt sidecars to the user recipient. Nodes already have separate persistent device age identities and ML-DSA signing keys, but there is no group identity bootstrap/distribution path.

Relevant prior context: [identity domain definition](2026-10-26-identity-domains.md), [node/app identity exploration](2026-10-22-node-and-app-age-identities.md), [encrypted config implementation](2026-10-17-rawrarr-age-encrypted-config.md).

## Design decisions
- The browser also receives the group private identity wrapped to the user age recipient, so it can continue decrypting group-encrypted metadata/config.
- Device envelopes are encrypted separately to each approved node's public device recipient. The coordinator transports ciphertext only.
- Group keys stay node-local after envelope installation; shared data stores only public recipient and recipient-specific ciphertext envelopes.
- Legacy user-recipient ciphertext remains readable during migration. New writes use the group recipient.

## Actions
- Added a persistent group age identity under node-local `/data/group-age-identity`. The public group recipient and a ciphertext copy wrapped to the user's age recipient are stored in synchronized shared data.
- Changed config and metadata writes to use the group recipient. Existing config can be read with the old user identity and is rewritten to the group identity; old metadata remains readable through a legacy-key fallback.
- Added a browser flow that downloads and decrypts the user's group-key envelope locally, encrypts the group identity to the selected node's device recipient, and installs that ciphertext on the node. New nodes therefore get a distinct device-encrypted envelope without the coordinator receiving plaintext key material.
- Wrapped user age unlock requests to the node's device recipient. The browser sends a base64-encoded age ciphertext; the node decrypts it with its node-local device age identity.
- Updated age setup and coordinator unlock flows, the generated browser bundle, integration fixtures, and test indexes.
- This is the first runtime slice of issue #109. Provisioning currently occurs when the owner unlocks a node, not automatically as part of node approval. Revocation, group/app key rotation, signed versioned membership transitions, stale rollback/fork prevention, and interrupted-rotation recovery remain unimplemented and must be completed before issue #109 can close.

## Verification
- `cargo test --locked --offline --manifest-path /workspace/rust-arr/Cargo.toml` in the isolated test image — 86 passed.
- Targeted coordinator/age UI tests in the isolated test image — 38 passed.
- `test_rawrarr_age_setup_browser.mjs` in the isolated test image — 1 passed.
- `npm run build` in `rust-arr/web-export` — passed; generated `rust-arr/src/assets/metadata-export.js`.
- `python3 -m py_compile scripts/tests/test_rawrarr_configuration.py scripts/tests/test_orbstack_arr_connectivity.py` — passed.
- `git diff --check` — passed.
- Final full regression rerun on the rebased commit passed: 110 Node tests, all Playwright browser tests, and 87 Rust tests. The container runner skipped Python integration tests that require disposable PostgreSQL, age, or Syncthing fixtures.
- Local macOS `cargo`/`cargo fmt` could not run because the installed LLVM requires a missing Z3 dylib. `rustfmt` is absent from the isolated image; formatting was not checked, per Will's instruction to prioritize merge readiness.
- GitHub PR [#139](https://github.com/willjasen/rawrarr/pull/139) merged to `main` at `6a74e332f46f84d78e6ae48b8ad7d2e6c0d0ada4`. The issue remains open for the lifecycle work below.
- No live-node or internal-environment test/deployment was run.

## Final state and remaining work
The device-to-group metadata encryption path is implemented and covered by the regression suite. The local node stores the group private ID, each device receives it encrypted to its device ID, the browser receives it encrypted to the user ID, and newly written metadata/config use the group recipient. Before closing issue #109, implement automatic approved-node provisioning, revocation and group/app key rotation, signed and versioned membership transitions coordinated with issue #40, stale rollback/fork rejection, offline/rejoin and interrupted-rotation recovery, and explicit tests for those transitions. Validate provisioning and revocation across two live test nodes before deployment. Revocation cannot erase keys, ciphertext, or plaintext already obtained by a revoked node.
