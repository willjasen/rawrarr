# Regression and browser coverage for implemented behavior gaps

Issue [#120](https://github.com/willjasen/rawrarr/issues/120).

## Objective

Add repeatable coverage for storage scan cancellation, the local Radarr plaintext mirror lifecycle, and the coordinator reconnect dialog.

## Initial state

Started from fetched `origin/main` at `b2fdc7a` in dedicated worktree `/Users/willjasen/.codex/worktrees/issue-120-regression-browser-coverage`, branch `codex/issue-120-regression-browser-coverage`. Existing source already implemented scan cancellation and local mirror lifecycle; the latter was exercised by `test_rawrarr_configuration.py` but absent from its test index description. Coordinator availability had pure monitor tests and a separate topology browser test covering reconnect, but no focused combined test.

## Actions

- Added a storage scan controller unit test for cancel rejection when inactive and cancellation request state while active.
- Added scan UI assertions for Stop visibility, POST routing, and pending stop feedback.
- Confirmed the existing topology browser test already exercises coordinator outage/recovery in Chromium; indexed that coverage and did not retain a duplicate standalone browser case.
- Updated the test index descriptions, including the already-runnable encrypted configuration integration test that covers saving the local Radarr plaintext mirror, restarting without an age identity, and production cleanup.

## Verification

- `git diff --check` passed.
- Focused Node tests for storage-scan UI and coordinator monitor logic passed (7/7).
- In the isolated test image, the existing topology Chromium test passed all 4 cases, including coordinator outage, pending dialog, and recovery. The Rust suite passed all 84 tests, including the new controller case. The full Node suite reported one pre-existing failure: `test_rawrarr_ui_signature_disabled.mjs` could not find `.github/workflows/tests.yml`, which is omitted by the runner snapshot.
- The local Radarr mirror lifecycle integration requires `RAWARR_TEST_DATABASE_URL` pointing to a disposable `_test` database and was not run separately.

## Final state and remaining work

Coverage and indexes are updated. The runner still reports the existing workflow-test failure because its snapshot omits `.github/workflows/tests.yml`; fix the runner snapshot or make that test skip when the workflow is unavailable. Run `test_rawrarr_configuration.py` against a disposable `_test` database when available. Run `test_rawrarr_configuration.py` against a disposable `_test` database when available. No application behavior or deployment was changed.
