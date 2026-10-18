# Add suggested account words

## Objective and initial state

Add the additional first-word and prehistoric-creature suggestions to the generated account-name pools. Existing pool-pair tests enforce the 32-character username limit.

Related context: [expand generated account vocabulary](2026-10-18-expand-generated-account-vocabulary.md).

## Changes

- Added ten era, material, and setting terms and ten prehistoric creatures.
- Kept all prefix/creature combinations within the existing username validation limit.

## Verification

- `node --test scripts/tests/test_rawrarr_passkey_ui.mjs scripts/tests/test_rawrarr_coordinator_workspace.mjs` passed all 16 checks.
- The isolated Rust account-name test passed and checked every pair across the expanded pools against the username limit.
- The roarlay release image built successfully and reported `healthy` after startup. HTTPS health identifies Roarlay, and Tailscale Serve JSON confirms HTTPS 443 `/` proxies to `http://127.0.0.1:8080`.
- `git diff --check` passed, and the roarlay deployment marker was released after verification.

## Final state

The previously suggested terms are now active in the generated-name pools. Because existing account display names are derived from account UUIDs and these pools, an account's displayed name can change after the vocabulary expands and the profile is refreshed.
