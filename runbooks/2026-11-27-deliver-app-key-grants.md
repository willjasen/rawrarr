# Deliver app-key grants to authorized fossils

Issue: [#500](https://github.com/willjasen/rawrarr/issues/500)

## Objective and initial state

Add an authenticated coordinator path for owner-issued, device-encrypted app-key grants. The coordinator must route opaque bundles only to the matching approved fossil and must not learn the app identity. Initial state: no grant API or storage exists. Related prerequisites: owner issuance [#497](https://github.com/willjasen/rawrarr/issues/497) is open; node verification [PR #494](https://github.com/willjasen/rawrarr/pull/494) is open and defines the current bundle schema.

## Design and adoption

Store the byte-exact signed JSON bundle in PostgreSQL, keyed by app instance, device, and key epoch. The coordinator validates bounded JSON structure and public routing fields but does not decrypt or re-sign. Identical retries are idempotent; a different bundle for an occupied epoch conflicts; lower epochs cannot supersede higher ones. Retrieval requires an authenticated metadata fossil, matching device ID and age recipient, and a currently approved node record. Revocation blocks node auth and removes pending grants. Data is new and additive; no existing state migration is needed beyond the new table. Rollback is to stop using the endpoints and drop the table only after confirming no clients rely on delivery.

The grant delivery API is app-type neutral. `appType` is signed routing metadata; no app-specific API calls or settings interpretation belong in the coordinator grant path. Radarr, Sonarr, Lidarr, and future general apps such as Duplicati share this identity and delivery boundary. Their API adapters and app lifecycles remain separate work.

## Progress

- Fetched `origin/main` at `06007b367a73ddf91a0b7f2c84ce0d2b1e15dc83` and created dedicated branch/worktree `codex/deliver-app-key-grants`.
- Preserved the unrelated untracked runbook in the shared checkout.
- Verified #497 and PR #494 remain open. Delivery can proceed independently using the verifier's current bundle shape; owner issuance remains unverified.

## Verification and remaining work

Added coordinator owner submission and authenticated fossil retrieval routes, migration `0013_app_key_grants.sql`, and signed node-proxy wiring for retrieving grants through a fossil. The retrieval assertion is included in the node-authenticated POST body, so upgraded signed requests cover the recipient value in their body signature. The store keeps one current grant per app/device, enforces root signature and ciphertext digest, and compares retries byte-for-byte.

Verification in an isolated `rawrarr-test-regression` Docker environment using a native `linux/amd64` image built from this worktree:

- `cargo test --locked --offline --manifest-path /workspace/rust-arr/Cargo.toml app_key_grant -- --nocapture` — 2 passed.
- `cargo check --locked --offline --manifest-path /workspace/rust-arr/Cargo.toml` — passed with one pre-existing `path_arg` dead-code warning.
- `git diff --check` — passed.
- `python3 scripts/tests/test_rawrarr_role_deployment.py --apply --app-key-grants-only` — passed against disposable PostgreSQL, a coordinator, metadata/storage fossils, and real Chromium. Verified approved owner setup, signed grant submission (`201`), identical retry (`204`), exact-byte and ciphertext-only persistence, authenticated exact delivery (`200`), anonymous/wrong-recipient/unknown-instance denial, owner submission and storage-role retrieval denial, and revocation deleting grants and preventing subsequent retrieval. The test image was built from this worktree under a unique local tag; no image was published.

The migration was moved from `0012_app_key_grants.sql` to `0013_app_key_grants.sql` after the disposable coordinator exposed a collision with the existing `0012_environment_owner.sql`; the harness passed after this correction.

The full role deployment harness was also attempted. The grant acceptance assertions passed before a stale Radarr URL fixture failed because it expected 400 for loopback HTTP. Issue [#514](https://github.com/willjasen/rawrarr/issues/514) corrected the fixture to distinguish rejected non-loopback HTTP (400) from unreachable loopback HTTP and HTTPS (502); its focused configuration run passed. The grant-only mode isolates this feature's acceptance path from later configuration fixtures.

As of this follow-up, owner grant creation [#497](https://github.com/willjasen/rawrarr/issues/497), node verification [PR #494](https://github.com/willjasen/rawrarr/pull/494), and managed-app settings integration [#499](https://github.com/willjasen/rawrarr/issues/499) remain open. This coordinator delivery layer is verified independently; deployment and full app settings use await those dependencies.

## General app acceptance follow-up

Issue #500 was clarified to require app-type-neutral grant delivery and multi-type acceptance coverage, coordinated with general app platform direction [#482](https://github.com/willjasen/rawrarr/issues/482) and settings integration [#499](https://github.com/willjasen/rawrarr/issues/499). The browser fixture and grant-only harness now submit and retrieve independent signed grants for `radarr`, `sonarr`, `lidarr`, `prowlarr`, and `duplicati`, verifying byte-exact ciphertext-only storage and revocation across all five. The Duplicati case exercises the generic grant contract only; it does not install Duplicati or test its APIs.

Focused multi-type verification passed after rebasing the task commit onto current `origin/main` and retaining the upstream WebRTC coordinator routes alongside the grant route:

- `python3 -m py_compile scripts/tests/test_rawrarr_role_deployment.py` — passed.
- `node --check scripts/tests/helpers/role_passkey_browser.mjs` — passed.
- `git diff --check` — passed.
- `docker --context orbstack build --platform linux/amd64 --build-arg RAWARR_BUILD_REVISION=3a5ea03 -t rawrarr:roles-general-3a5ea03-amd64 -f Dockerfile .` in `rust-arr/` — passed; embedded revision and architecture read back as `3a5ea03` / `amd64`. The image is local-only and was not published.
- `python3 scripts/tests/test_rawrarr_role_deployment.py --apply --app-key-grants-only --image rawrarr:roles-general-3a5ea03-amd64 --browser-image rawrarr-testing-base:local` — passed against that image. The browser owner signed independent grants for `radarr`, `sonarr`, `lidarr`, `prowlarr`, and `duplicati`; all were idempotently submitted, stored byte-exactly without plaintext app identities, retrieved byte-exactly through the same metadata-fossil endpoint, and removed/blocked after fossil revocation. The Duplicati entry is a generic contract fixture only, not an installed app or tested API adapter.
