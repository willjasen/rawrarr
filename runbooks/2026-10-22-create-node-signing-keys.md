# Create persistent node signing keys

## Objective

Give each subscriber node a local ML-DSA-65 signing key, separate from the user's signing authority, created on startup when absent and reused across restarts. Track implementation under [GitHub issue #21](https://github.com/willjasen/rawrarr/issues/21).

## Initial state

The browser has a client-side ML-DSA-65 instance-root key primitive, but subscriber startup only creates a separate age identity. The user requested treating the existing root authority as the user signing key and adding per-node signing keys. The shared checkout had concurrent staged and unstaged changes and was 11 commits behind `origin/main`; work was isolated in a dedicated worktree based on `origin/main`.

GitHub issue search through the connector returned an access validation error. `gh issue list` found no open matching issue, so issue #21 was created before implementation with acceptance criteria for generation, persistence, restrictive permissions, invalid-file preservation, and coordinator exclusion.

## Planned implementation

- Add subscriber-startup creation and validation of `/data/node-signing-key` using ML-DSA-65.
- Store only the 32-byte private seed in a versioned file envelope, atomically and with mode `0600`; keep it separate from `/data/age-identity` and `/data/node-age-identity`.
- Reuse valid existing key material. Leave invalid existing files untouched and fail startup.
- Document that enrollment, public-key publication, and user-key authorization remain future work.

## Verification and final state

Implemented startup creation of a versioned ML-DSA-65 seed file at `/data/node-signing-key`, using atomic create-new plus hard-link publication and mode `0600`. Subscribers now ensure the signing key after the early coordinator exit, alongside their separate node age identity. Existing keys are parsed/reused, and unsupported files fail without replacement. Tests cover stable public-key derivation across reload, file reuse, file permissions, user-identity separation, and invalid-file preservation. The setup flow now calls the existing authority key a user signing key; encrypted backup bytes and its legacy format are unchanged.

Verification:

- Rust `cargo check` passed using the installed Rust 1.85.1 toolchain directly; the default Homebrew rustc shim aborts because its LLVM dependency requests unavailable Z3 5.1.
- Focused `cargo test --locked --manifest-path rust-arr/Cargo.toml node_signing_key -- --nocapture` passed (2 tests).
- The coordinator workspace's selected user-signing-copy test passed (1 test), the subscriber onboarding test file passed `node --check`, and `git diff --check` passed.
- The onboarding bundle rebuilt with the locked npm dependencies. The full coordinator workspace test file was also attempted; 7 existing assertions fail against this `origin/main` baseline, while the updated user-signing-copy assertion passes. Repository-wide `cargo fmt --check` reports formatting diffs throughout unrelated existing files, so it was not used to reformat broad source.
- No deployment or live-node key creation was performed. Generated private seeds are not enrolled or used for signatures yet. The selected RustCrypto ML-DSA crate documents that it has not been independently audited; cryptographic review remains required before production signing.

Changes are committed on `codex/node-signing-key`; they have not been pushed or merged. No deployment is included in this task.
