# Require age identity backup confirmation

## Objective and initial state

The user wants setup to require acknowledgement that the private age identity is saved somewhere recoverable before vault unlock. Previously, the age setup screen allowed unlock immediately after identity creation or import.

## Changes

- Show an unchecked backup confirmation once a generated, pasted, or file-loaded identity is available.
- Keep **Unlock vault** disabled until the confirmation is checked, and enforce the same condition in its click handler.
- Reset the confirmation when a new identity is generated, typed, or loaded so the acknowledgement applies to the current key.
- Explain that Rawrarr cannot recover a lost identity; update the design guide and user instructions.
- Add regression coverage for generated and file-loaded identities.

## Verification

`node --test scripts/tests/test_rawrarr_age_setup.mjs` passes (1/1), and `git diff --check` is clean. Rebuilt and restarted `rawrarr-local-rawrarr-1` in OrbStack; it is running with `127.0.0.1:8084->8080`. `/health` returns `ok`. `/setup/age` returns 401 without an authenticated session, so live page rendering was not verified; the DOM harness covers the gated flow. Do not reload an open age setup tab that may still hold a newly generated identity in page memory.
