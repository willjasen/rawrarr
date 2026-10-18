# Derive Rawrarr's theme from the age identity

## Objective and initial state

The age setup screen already showed a colored fingerprint derived from the public age recipient, but the Rawrarr workspace kept a fixed green palette. The user wants the app palette to become identity-specific after a public age identity is created or imported.

## Changes

- Derive a SHA-256 theme seed from the public recipient in the backend. Return only the seed with generated/unlocked age status; the status endpoint does not expose the recipient itself.
- Apply the same deterministic hue on the age setup screen after generation or successful unlock, and load it into the main workspace after an unlocked status check.
- Use that hue for the app's dark surface tint, accent, active navigation, and welcome panel while retaining readable text and semantic status colors.
- Extend the age setup and configuration checks, and update the design guide.

## Verification

- `node --test scripts/tests/test_rawrarr_age_setup.mjs` passed (1 test); it verifies generated-identity theme application, privacy-safe recipient display, and file-load/unlock separation.
- `git diff --check` passed.
- OrbStack rebuilt Rawrarr successfully, which compiled the Rust changes with `cargo build --release --locked`; `/health` returned `ok` and PostgreSQL remained healthy.
- The local `cargo test --locked` command could not start because the host Rust toolchain's LLVM library cannot load its expected Z3 dylib. The existing API integration script was not run because no disposable PostgreSQL `_test` database is configured.
- The main UI obtains a derived theme seed from the authenticated age-status API after unlock. Interactive browser verification requires signing in with the passkey.
