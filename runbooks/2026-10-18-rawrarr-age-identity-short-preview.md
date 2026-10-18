# Show a shortened age identity preview

## Objective and initial state

Show a recognizable shortened form after generating an age identity while keeping the full private identity available only through the explicit copy action. The earlier copy-only screen kept the full key out of rendered elements; see [the prior screen work](2026-10-18-rawrarr-age-identity-copy-screen.md). The user asked for a short preview without displaying the complete identity.

## Changes

- Added an accessible preview card to the age setup screen. It extracts the secret-key line and displays only the first 12 and last 5 characters separated by an ellipsis.
- The complete generated identity remains in page memory and is still available only to the copy button or the unlock request. Importing an identity hides/clears the generated preview.
- Updated `scripts/tests/test_rawrarr_age_setup.mjs` and its index description to assert the exact preview, no full identity in page elements, clipboard copy of the full fixture, and successful unlock request handling.

## Verification and final state

`node --test scripts/tests/test_rawrarr_age_setup.mjs` passed (1 test). Rebuilt and recreated only `rawrarr-local-rawrarr-1` in OrbStack with `--no-deps`; the existing PostgreSQL container and volume were left running. `http://127.0.0.1:8084/health` returned `ok`, and the app container is running on port 8084. The UI test verifies the abbreviated preview and proves the complete fixture is absent from rendered elements while the copy action receives it. The preview intentionally exposes a small portion of the private key at the user's request; the complete identity remains unrendered.
