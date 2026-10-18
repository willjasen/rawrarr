# Keep private age identity off the setup screen

## Objective and initial state

The owner reported that newly generated private age identity text was displayed on the setup screen and requested a copy-only flow plus a visual mark derived from the public identity. Rawrarr was running in OrbStack at `http://localhost:8084`; the earlier setup form used a visible text area. Related context: [age identity onboarding](2026-10-17-rawrarr-age-identity-onboarding.md) and [shared age config](2026-10-17-rawrarr-shared-age-config.md).

## Changes

- Generated private identity remains in page memory and is available only through the explicit copy action. The page does not put it into rendered text or a visible form field.
- Existing identities can be pasted into a password-masked field or selected from a local file; file contents are placed only in the masked field.
- The public age recipient is displayed with a deterministic mirrored-square fingerprint derived from its SHA-256 digest.
- The page clears its in-memory/generated and masked identity values when unlocking or leaving the page.
- Added a Node DOM-harness test to verify the private key is not rendered, copying yields the private value only to the clipboard API, and the public fingerprint is rendered.

## Verification

- `node --test scripts/tests/test_rawrarr_age_setup.mjs` passed.
- Changed Rust vault module formatting check passed.
- Rebuilt and restarted only `rawrarr-local-rawrarr-1`; PostgreSQL and its named volume remained in place.
- OrbStack app health returned `ok`. Through the existing authenticated browser session, `/setup/age` displayed the masked import field, copy-only action, public recipient fingerprint section, and no rendered private identity.
- Direct unauthenticated access to `/setup/age` returned HTTP 401, as expected.

## Final state and remaining work

The updated setup screen is running at `http://localhost:8084/setup/age`. Because the server restart clears the in-memory age unlock, sign in and import the backed-up identity again to unlock the vault. The app's generated identity is never persisted by the server. Keep secret contents out of logs, screenshots, and test output.
