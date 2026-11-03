# Show the browser age public ID

Issue: [#311](https://github.com/willjasen/rawrarr/issues/311)

## Behavior

Settings has a hidden-by-default button in the “Age private ID in this browser” card. When requested, the coordinator derives and validates the public recipient locally from the browser's saved age private ID, displays only an abbreviated recipient, and offers a separate Copy button for the complete public value. Hiding it, clearing the saved age ID, or leaving the page removes the displayed and in-memory values. The derivation uses the existing browser age-encryption bundle; the private identity is not sent to the coordinator for this display.

## Compatibility and adoption

This is a coordinator UI-only change. It adds no persistent data, database schema, encryption format, or protocol changes, so there is no migration. Existing deployments adopt the feature through their normal coordinator UI release.

## Verification

- `node --test scripts/tests/test_rawrarr_coordinator_workspace.mjs` — passed (34 tests).
- `python3 scripts/tests/run_rawrarr_tests_in_docker.py` — passed Node (146), Playwright, and Rust (113) suites for the initial reveal/hide control.
- Follow-up `node --check rust-arr/src/coordinator-workspace.mjs` and `node --test scripts/tests/test_rawrarr_coordinator_workspace.mjs` — passed (34 tests).
- Follow-up focused Chromium `test_rawrarr_coordinator_sonarr_settings.mjs` — passed (6 tests), including copy/redaction assertions and 1280×800 / 375×812 layout checks. The broader Docker run passed Node (146) and Rust (115); one unrelated `test_rawrarr_signing_topology_browser.mjs` assertion (`desktop section title is visible`) failed, and Python integration was skipped.

## Deployment

- Merged as `8ff9e81c52d07e0a4c5bcba41c0059b9ef437c96` and deployed from its authenticated GitHub archive to do3's public `rawrarr-roarlay-prod` `roarlay` service only.
- The native `linux/amd64` image is `rawrarr:8ff9e81c52d07e0a4c5bcba41c0059b9ef437c96-amd64`, image ID `sha256:a1a076f430c29e414485c28f99e9db2d773dfc0cb9003a61d04aca12018da7bb`. Container health is healthy with restart count 0; `https://rawrarr.willjasen.net/health` returned HTTP 200 and the public workspace script exposed the shortened-ID/copy implementation.
- Preserved the prior service image and reusable base under `rawrarr:rollback-public-before-8ff9e81-amd64` and `rawrarr:rollback-base-before-8ff9e81-amd64`. The internal coordinator and subscriber nodes were not changed.
