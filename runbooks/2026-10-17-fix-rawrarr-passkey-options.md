# Fix Rawrarr passkey option decoding

## Objective and initial state

Fix the sign-in error `Cannot read properties of undefined (reading 'replace')` reported on the passkey page. The existing [passkey onboarding runbook](2026-10-17-rawrarr-passkey-postgres-onboarding.md) documents the auth flow.

## Findings and changes

- The Rust WebAuthn response serializes options as `{ publicKey: { challenge, ... } }`, while the page attempted to read `challenge` directly from the wrapper.
- Extracted the browser-safe option decoder into `rust-arr/src/passkey-client.mjs`; it decodes the nested `publicKey` object and reports malformed server responses clearly.
- Served the module from Rawrarr and changed the passkey page to use it.
- Added a Node test for the nested challenge, user ID, credential IDs, and malformed payload error.

## Verification and remaining work

`node --test scripts/tests/test_rawrarr_passkey_ui.mjs` passed (2 tests). The OrbStack image was rebuilt; `/health` returns HTTP 200, `/auth` serves the passkey page, `/assets/passkey-client.mjs` serves the decoder, and a live registration-start response contains `options.publicKey.challenge` with the configured `localhost` RP ID. The temporary challenge was deleted. The fixed passkey page is open at `http://localhost:8084/auth`.

The prior native page on port 8082 currently returns HTTP 404 and was left untouched. The user's actual authenticator creation/login ceremony has not been completed; the user can retry on the updated Docker page.
