# Show the browser age public ID

Issue: [#311](https://github.com/willjasen/rawrarr/issues/311)

## Behavior

Settings has a hidden-by-default button in the “Age private ID in this browser” card. When requested, the coordinator derives and validates the public recipient locally from the browser's saved age private ID, displays only an abbreviated recipient, and offers a separate Copy button for the complete public value. Hiding it, clearing the saved age ID, or leaving the page removes the displayed and in-memory values. The derivation uses the existing browser age-encryption bundle; the private identity is not sent to the coordinator for this display.

## Compatibility and adoption

This is a coordinator UI-only change. It adds no persistent data, database schema, encryption format, or protocol changes, so there is no migration. Existing deployments adopt the feature through their normal coordinator UI release.

## Verification

- `node --test scripts/tests/test_rawrarr_coordinator_workspace.mjs` — passed (34 tests).
- `python3 scripts/tests/run_rawrarr_tests_in_docker.py` — passed Node (146), Playwright, and Rust (113) suites for the initial reveal/hide control.
- Follow-up shortened-display and copy coverage: pending.
