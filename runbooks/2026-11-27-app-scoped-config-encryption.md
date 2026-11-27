# App-scoped configuration ciphertext

Issue [#110](https://github.com/willjasen/rawrarr/issues/110).

## Objective

Verify owner-root-signed app-key grants that wrap each app identity to an authorized node's age recipient, persist app-key epochs under `/local`, and stage app-owned JSON ciphertext under `/evt/fossil`. This remains preparatory and does not activate settings replication.

## Initial state

Started from a clean dedicated worktree, `codex/app-scoped-config-encryption`, created from `origin/main` at `a318db5`. Existing code prepares Radarr/Sonarr app identities in group-encrypted `/shared/app-identities.aged`; all nodes able to unlock the group key can read those keys. There was no app-recipient config ciphertext path.

## Implementation

- Added a 2 MiB-bounded JSON object writer/reader to `AgeVault`.
- A grant verifier checks the pinned instance root fingerprint and ML-DSA signature, binds app instance/type, device ID/recipient, key epoch, and envelope digest, decrypts only with this node's local age identity, and stores the app key under `/local/app-identities/<app-instance-id>/epoch-<n>/identity`. App JSON ciphertext stages at `/evt/fossil/apps/<app-instance-id>/config.aged`; path IDs must be canonical UUIDs. Local state prefers `RAWARR_LOCAL_DIR` and retains `RAWARR_DATA_DIR` as a compatibility fallback. The event root can be set with `RAWARR_EVT_DIR` and defaults beside the shared mount (container `/evt`).
- Parent directories are checked as non-symlink directories. Reads use the existing bounded regular-file reader, require age ciphertext, decrypt only with the matching app identity, validate the JSON object, and return plaintext in a zeroizing buffer.
- The focused Rust test creates a real ML-DSA-signed grant and age envelope, verifies successful install, rejects wrong-device and invalid-epoch grants, checks private file permissions, then proves round-trip and different-app decryption denial. It also checks invalid IDs/payloads and symlinked event-domain directories.
- Updated the test index and identity/data-domain docs.

This primitive is deliberately not connected to an app settings endpoint. Prepared app identities remain in a group-encrypted registry, but the grant verifier does not use that registry as its delivery source. There is no owner-side grant issuer, endpoint to deliver grants, cross-node writer protocol, enforced revocation or key rotation workflow, or migration. Do not activate production app configuration replication until those gates are met.

## Verification

Used a uniquely named `rawrarr-test-regression` container in OrbStack, copied this worktree's Rust source and tests, prepared dependencies while networked, disconnected it from the bridge, and ran focused Rust tests offline:

- `cargo test --locked --offline --manifest-path /workspace/rust-arr/Cargo.toml prepared -- --nocapture` — 2 passed.
- `cargo test --locked --offline --manifest-path /workspace/rust-arr/Cargo.toml secure_config::tests::encrypted_config_and_artwork_are_shareable_age_ciphertexts -- --exact` — 1 passed.
- `git diff --check` — passed.

Focused checks in the network-disconnected OrbStack clone passed:

- `cargo test --locked --offline --manifest-path /workspace/rust-arr/Cargo.toml staged_app_configuration -- --nocapture` — 1 passed.
- `cargo test --locked --offline --manifest-path /workspace/rust-arr/Cargo.toml prepared -- --nocapture` — 2 passed.
- `cargo test --locked --offline --manifest-path /workspace/rust-arr/Cargo.toml secure_config::tests::encrypted_config_and_artwork_are_shareable_age_ciphertexts -- --exact` — 1 passed.
- `cargo test --locked --offline --manifest-path /workspace/rust-arr/Cargo.toml owner_recipient -- --nocapture` — 3 passed.
- `git diff --check` — passed.

The second test-clone preparation attempt reported an apt `/tmp` permission error, but dependency preparation had already completed before the clone was disconnected; the focused Rust tests ran offline. The whole-crate `cargo fmt --check` was attempted earlier on the host and reports extensive pre-existing formatting differences across included test modules; no broad formatting was applied. The full Rust, Node, Playwright, and environment suites were not run.

## Adoption, rollback, and remaining work

No deployment adopts this path yet, and no data/protocol migration is required because no runtime route calls it. Rollback removes the new primitive/docs/test. Before activation, implement owner-root-signed app-to-device grants, deliver app keys age-encrypted to each authorized device recipient, verify grants against the pinned root and device identity, define offline/revocation/rotation recovery, and establish conflict-safe writers and versioned readers. Preserve old group-encrypted config reads during migration and fence old writers before any ciphertext rewrite.

## Final state

The branch verifies owner-root-signed, node-wrapped app-key grants and keeps app keys in node-local epoch storage. App JSON ciphertext stages under `/evt/fossil`. No owner-side grant issuer, delivery endpoint, runtime settings flow, or authorized RPC replication path is active. The helper is not called by production code and does not constitute completed replication.
