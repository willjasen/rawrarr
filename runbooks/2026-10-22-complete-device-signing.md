# Complete per-instance device signing

## Objective

Resolve [GitHub issue #37](https://github.com/willjasen/rawrarr/issues/37): finish owner-rooted ML-DSA device enrollment, node authorization, signed shared records, and a non-destructive migration for deployed nodes.

## Initial state

GitHub `origin/main` at `97481c5` already contains the browser root-key primitive, subscriber onboarding preview, and persistent per-node ML-DSA key generation (PRs #25 and #26). Its design note explicitly says enrollment, public-key publication, root authorization, signed shared records, and migration are not implemented. The shared checkout is 31 commits behind `origin/main` and contains staged/unstaged changes in unrelated work, so this task is isolated at `/private/tmp/rawrarr-device-signing-37` on `codex/instance-device-signing`.

## Plan

1. Define the versioned protocol and genesis root pinning. Retain existing node IDs and roles; derive cryptographic IDs separately.
2. Complete browser setup and owner approval while keeping age/root plaintext in active-tab memory only.
3. Add root-signed, replay-resistant approvals, server verification, device proof of possession, and node request signatures while keeping a clearly identified legacy migration path.
4. Preserve the public trust registry through encrypted-config updates and implement signed `/shared` record envelopes with verification and unsigned-legacy behavior.
5. Add Rust/browser interoperability and failure-mode tests, verify the full path, document limitations and the migration, then close the issue only if every criterion is met.

## Actions and verification

Implemented the first end-to-end signing increment:

- Added the versioned `ML-DSA-65` verifier, canonical JSON signing profile, and root certificate checks.
- Added additive migration `rust-arr/migrations/0008_instance_device_signing.sql`; old node rows remain token-only until explicit owner approval.
- Added proof-of-possession enrollment, browser-side root approval, root fingerprint verification, signed coordination requests, and durable replay-nonce rejection.
- Added signed, compressed operational records in `/shared/logs` with a root-signed node certificate and a trusted-record verifier.
- Changed the owner age identity and root approval form to volatile page memory; updated onboarding to require a root fingerprint pin and an explicit backup acknowledgement/warning.
- Updated the test indices and design documentation with the safe upgrade sequence and limits. See [device signing design](../docs/device-signing-design.md).

Focused checks run in the isolated worktree:

| Check | Result |
| --- | --- |
| `cargo test --bin rust-arr` with `DYLD_LIBRARY_PATH` and `DYLD_FALLBACK_LIBRARY_PATH` pointed at `/tmp/rawrarr-z3-compat`, and the Rust 1.90 compiler selected explicitly | 72 passed, 0 failed |
| `node --test scripts/tests/test_rawrarr_device_signing_crypto.mjs scripts/tests/test_rawrarr_subscriber_compose.mjs` | 5 passed, 0 failed |
| Focused volatile-approval browser-module test | 1 passed, 0 failed |
| `node --check` for coordinator workspace and subscriber onboarding bundle source | passed |
| `git diff --check` | passed |

Formatting was applied only to the new signing module and directly modified small modules; project-wide `cargo fmt --all -- --check` remains known to fail on pre-existing formatting differences, so do not format the entire crate. The focused volatile approval module test passed. The Playwright age-memory tests could not start because the host's configured `/usr/local/lib/node_modules/playwright-core/index.mjs` is missing. No deployment was performed.

Migration remains additive and is documented above. Before deploying, back up coordinator DB and persistent node data. Upgrade coordinator first, confirm 0008 is applied, then regenerate each subscriber Compose with the existing IDs/token file/volumes plus the independently checked fingerprint and enrollment enabled. Approve each retained node key in the browser using the age identity and encrypted root backup. Keep old binaries token-only during staged rollout; do not claim them as signature-verified. Roll back by restoring DB and image/config together. No node IDs, credentials, or data volumes are reset by this migration.

Implementation merged to GitHub `main` in PR [#41](https://github.com/willjasen/rawrarr/pull/41), merge commit `3b5ee1fe1d74b50cfb7a42460b6b47863ce3b965`. GitHub issue [#37](https://github.com/willjasen/rawrarr/issues/37) is resolved; its broader registry, general signed-artifact, revocation/rollback, and cross-language-vector scope is tracked in follow-up issue [#40](https://github.com/willjasen/rawrarr/issues/40). Root rotation/recovery and independent cryptographic review remain unimplemented. No production deployment was performed.

Review found and fixed an upgrade blocker before merge: a legacy node requesting approval with its existing ID was rejected merely because that ID already existed. Enrollment now permits reuse only when both the existing token digest and role match; collisions with another credential or role still return conflict. The Rust enrollment test suite asserts this in-place upgrade condition.

## Remaining work

- Complete code/security review and confirm all checks against the updated commit.
- Finish CI/browser validation where the required Playwright installation is available.
- Create a PR, merge only after the review is clean, then use GitHub as source of truth and clean up the task worktree/branch.
- Production deployment is separate and has not been authorized in this task.
