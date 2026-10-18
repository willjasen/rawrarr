# Show the rawrarr logo on setup

## Objective

Show the same rawrarr logo on the age identity setup page that appears in the main workspace.

## Initial state

The setup page showed a text brand label but omitted the existing `/assets/rawrarr-logo.png` image. The app already served the asset for the dashboard.

## Changes

- Added the existing logo asset above the setup page heading, with lowercase alternative text and dashboard-matching treatment.
- Added a page test to keep the logo reference present.

## Verification

- `node --test scripts/tests/test_rawrarr_age_setup.mjs` passed with the logo reference assertion.
- Docker image rebuild succeeded; `/health` returned `ok` and both containers report `healthy`.
- `http://localhost:8084/assets/rawrarr-logo.png` returned HTTP 200 (662823 bytes).

## Final state

The age setup page references the same logo served to the dashboard, with lowercase alternative text.
