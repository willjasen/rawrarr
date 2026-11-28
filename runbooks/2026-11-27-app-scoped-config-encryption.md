# App-scoped configuration ciphertext

Issue [#110](https://github.com/willjasen/rawrarr/issues/110).

## Objective

Add a bounded storage primitive that encrypts app-owned JSON configuration with the prepared app instance's age identity, without changing active `config.aged` or metadata behavior. This is groundwork for future authorized app-config replication.

## Initial state

Started from a clean dedicated worktree, `codex/app-scoped-config-encryption`, created from `origin/main` at `a318db5`. Existing code prepares Radarr/Sonarr app identities in group-encrypted `/shared/app-identities.aged`; all nodes able to unlock the group key can read those keys. There was no app-recipient config ciphertext path.

## Implementation

- Added a 2 MiB-bounded JSON object writer/reader to `AgeVault`.
- The writer uses the prepared app recipient and stores ciphertext at `/shared/in/apps/<app-instance-id>/config.aged`; path IDs must be canonical UUIDs.
- Parent directories are checked as non-symlink directories. Reads use the existing bounded regular-file reader, require age ciphertext, decrypt only with the matching app identity, validate the JSON object, and return plaintext in a zeroizing buffer.
- Added a focused Rust test for exact round-trip, ciphertext/plaintext separation, rejection by a different app identity, invalid IDs/payloads, and symlinked app directories.
- Updated the test index and identity/data-domain docs.

This primitive is deliberately not connected to an app settings endpoint. The app private keys are still stored in a group-encrypted registry, so it does not create an app authorization boundary. No cross-node writer protocol, signed per-app grants, device-wrapped key delivery, revocation/rotation, or migration is implemented. Do not activate production app configuration replication until those gates are met.

## Verification

Used a uniquely named `rawrarr-test-regression` container in OrbStack, copied this worktree's Rust source and tests, prepared dependencies while networked, disconnected it from the bridge, and ran focused Rust tests offline:

- `cargo test --locked --offline --manifest-path /workspace/rust-arr/Cargo.toml prepared -- --nocapture` — 2 passed.
- `cargo test --locked --offline --manifest-path /workspace/rust-arr/Cargo.toml staged_app_configuration -- --nocapture` — 1 passed.
- `cargo test --locked --offline --manifest-path /workspace/rust-arr/Cargo.toml secure_config::tests::encrypted_config_and_artwork_are_shareable_age_ciphertexts -- --exact` — 1 passed.
- `git diff --check` — passed.

The whole-crate `cargo fmt --check` was attempted on the host and reports extensive existing formatting differences across included test modules; no broad formatting was applied. The full Rust, Node, Playwright, and environment suites were not run.

## Adoption, rollback, and remaining work

No deployment adopts this path yet, and no data/protocol migration is required because no runtime route calls it. Rollback removes the new primitive/docs/test. Before activation, implement owner-root-signed app-to-device grants, deliver app keys age-encrypted to each authorized device recipient, verify grants against the pinned root and device identity, define offline/revocation/rotation recovery, and establish conflict-safe writers and versioned readers. Preserve old group-encrypted config reads during migration and fence old writers before any ciphertext rewrite.

## Final state

App-scoped JSON ciphertext can be staged and locally read with its matching prepared app identity. It is not yet authorized or consumed by the application and must not be described as completed replication.
