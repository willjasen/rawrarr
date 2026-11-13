# Patch WebAuthn dependency advisory

## Objective

Resolve Dependabot alert #2 (`GHSA-22w3-693w-x895`) by upgrading the vulnerable
WebAuthn dependency from 0.5.4 to the first patched stable release, 0.5.5.
Tracking issue: #464.

## Scope and compatibility

- Upgrade `webauthn-rs` and its `webauthn-rs-core` dependency, and keep the lock
  file on the patched release.
- Raise the Rust minimum and production/test builder images to Rust 1.88,
  required by `webauthn-rs` 0.5.5.
- No persistent data, database schema, encrypted configuration, or node
  protocol changes; no migration is required. Existing deployments adopt the
  fix when rebuilt from the released application source/image. Rollback is a
  normal application-image rollback and does not require data restoration.

## Actions

- Updated `webauthn-rs` to 0.5.5 and refreshed its locked transitive WebAuthn
  crates, including `webauthn-rs-core`.
- Raised the `rust-arr` minimum and production/test builder toolchains to Rust
  1.88, and aligned the Rust development/testing documentation.
- The reusable `rawrarr-testing-base:local` image must be refreshed when
  adopting this toolchain baseline with
  `python3 scripts/tests/run_rawrarr_tests_in_docker.py --rebuild-base`.

## Verification

- Passed `cargo check --locked --manifest-path rust-arr/Cargo.toml` using Rust
  1.89.0.
- Passed `cargo test --locked --manifest-path rust-arr/Cargo.toml
  auth::tests::account_names_are_dinosaur_themed_and_unique_per_account_id`
  using Rust 1.89.0 (1 passed).
- Confirmed `cargo tree --locked --manifest-path rust-arr/Cargo.toml -i
  webauthn-rs-core` resolves `webauthn-rs-core` 0.5.5 through
  `webauthn-rs` 0.5.5.
- `git diff --check` passed.
- The complete Rust, Node.js, and Playwright suites and Docker image builds were
  not run.
