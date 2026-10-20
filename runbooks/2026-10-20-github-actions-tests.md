# GitHub Actions test workflow

## Goal

Run the repository's existing Node.js, Playwright/Chromium, and Rust test suites automatically on pull requests and changes to `main`.

## Implementation

- Added `.github/workflows/tests.yml` for pull requests, pushes to `main`, and manual dispatch.
- The workflow builds the existing isolated test image and mounts only the checked-out source into the test container.
- The test container runs with networking disabled, matching the local runner's isolation boundary.
- The workflow requests read-only repository contents and does not require secrets.
- Python database-backed and host Docker/OrbStack integration tests remain out of CI because they require separate disposable fixtures.

## Verification

Built the CI test image and ran the workflow's test-container command locally. The Playwright/Chromium suite passed, but the existing aggregate runner exited nonzero: the Node.js suite reported multiple assertion/runtime failures, and the Rust suite reported five failing tests in direct-route validation, avatar encoders, and quality-profile caching. These failures are not caused by workflow edits; they should be triaged separately. The workflow itself must run on GitHub Actions to verify the hosted runner environment.

## Files

- `.github/workflows/tests.yml`
- `TESTING.md`
- `runbooks/README.md`
