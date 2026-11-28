# First account sign-in with an ENS wallet

Issue: [#527](https://github.com/willjasen/rawrarr/issues/527) (slice of [#445](https://github.com/willjasen/rawrarr/issues/445))

## Objective and initial state

Allow a person without an existing passkey or Rawrarr account to establish the initial coordinator account by proving control of an ENS name's resolved Ethereum address. Existing wallet login only accepted an address already linked to an account. The shared checkout had unrelated worktree state; implementation is isolated in `codex/ens-wallet-first-account` from fetched `origin/main` (`b78257a`).

## Changes

- The existing ENS login start route issues a distinct `wallet_signup` challenge only when the address is unlinked and the coordinator has no users. Linked wallets retain the existing `wallet_login` flow; an unlinked address is still rejected when any account already exists.
- Finish re-resolves the ENS name and verifies the SIWE signature, then serializes with passkey registration under the existing account-creation advisory lock. In one transaction it requires the user table and wallet mapping to remain empty, creates a generated account and wallet association, assigns unowned nodes as first-account setup does, consumes the challenge, and issues the normal authenticated session.
- Migration 0013 allows the new challenge flow while preserving the invariant that only signup challenges have no account ID. Existing auth challenges remain compatible. The existing coordinator auth path hands the new account into age ID and signing-root setup.
- No private wallet material is requested or stored. No adoption action beyond the normal application migration at startup is needed. On rollback, migration constraints remain compatible with the prior binary; the old code ignores `wallet_signup` challenges and cannot complete their flow. No data rollback is required.
- The separate collectible-style ENS artwork in issue #445 is outside this slice and remains unimplemented.

## Verification

- `node --test scripts/tests/test_rawrarr_passkey_ui.mjs` — passed (7 tests).
- `cargo check --locked --manifest-path rust-arr/Cargo.toml` — passed; emitted existing dead-code warnings.
- `cargo test --locked --manifest-path rust-arr/Cargo.toml coordinator_crypto::tests:: -- --nocapture` — passed (9 tests).
- `git diff --check` — pending final review.
- PostgreSQL-backed API test and live ENS/SIWE browser flow — not yet run.

## Final state

Implementation, database integration coverage and PR review remain in progress.
