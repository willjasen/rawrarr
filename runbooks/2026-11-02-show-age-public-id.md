# Show the browser age public ID

Issue: [#311](https://github.com/willjasen/rawrarr/issues/311)

## Behavior

Settings now has a hidden-by-default button in the “Age private ID in this browser” card. When requested, the coordinator derives and validates the age public recipient locally from the browser's saved age private ID, then displays the public recipient in selectable text. Hiding it or clearing the saved age ID removes the value from the page. The derivation uses the existing browser age-encryption bundle; the private identity is not sent to the coordinator for this display.

## Compatibility and adoption

This is a coordinator UI-only change. It adds no persistent data, database schema, encryption format, or protocol changes, so there is no migration. Existing deployments adopt the feature through their normal coordinator UI release.

## Verification

- `node --test scripts/tests/test_rawrarr_coordinator_workspace.mjs` — passed (34 tests).
- `python3 scripts/tests/run_rawrarr_tests_in_docker.py` — passed Node (146), Playwright, and Rust (113) suites. Chromium exercised the new age ID flow, including reveal, hide, private-ID redaction, and 1280×800 / 375×812 viewport checks.
