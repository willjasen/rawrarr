# Hide unavailable ENS status

## Objective

Hide the initial account-tier and ENS-wallet status messages in Settings until crypto account and wallet data load. Tracked by [issue #441](https://github.com/willjasen/rawrarr/issues/441).

## Initial state

- Started from fetched `origin/main` (`a8a1995`) in a dedicated worktree.
- The Settings template rendered both status messages immediately. `loadProfile()` replaced them only after both crypto API requests succeeded, so they remained visible while the browser had no injected wallet provider and requests were pending or failed.

## Implementation

- Mark both crypto status paragraphs hidden in the initial HTML.
- Reveal them only after account-tier and linked-wallet requests resolve, preserving the existing resolved text and link-wallet form.
- This is a static coordinator UI change with no persisted state, API, schema, encrypted-config, or node-protocol impact; no migration is required. Existing deployments adopt it with the updated coordinator UI.

## Verification

- Focused `test_rawrarr_crypto_ui.mjs` run in an isolated OrbStack container using `rawrarr-testing-base:local`: passed (15 tests, 0 failures).
- `git diff --check`: passed.

## Final state

- The initial loading placeholders remain hidden until both crypto API responses resolve. The resolved account tier and wallet status still render; the wallet-link form remains available.
- Branch: `codex/hide-unavailable-ens-status`. No deployment performed.
- No persistent-data, API, schema, or node-protocol migration is needed.
