# Regression fixture and signing topology checks

Issue: https://github.com/willjasen/rawrarr/issues/370

## Scope and root causes

- The reported Docker fixture failure expected the old literal `ROOT / "docs"`
  copying implementation. Current main already checks the `ROOT_FIXTURES`
  manifest instead, following the fixture-copy refactor. No additional fixture
  implementation change is needed.
- The signing topology section follows the device cards and network graph.
  Its test incorrectly required the section title to be above the fold at page
  load. A rendered section can correctly require vertical scrolling at both
  desktop and mobile sizes.

## Change

Scroll the signing topology heading into view before measuring its position.
Require its full rectangle to fit the viewport, including positive dimensions
and both horizontal and vertical bounds. Retain the root/device card width,
long-label, and document overflow checks at 1280x800 and 375x812.

## Verification

All regression runs use
`python3 scripts/tests/run_rawrarr_tests_in_docker.py`, copying the local
working tree into a new linked clone without rebuilding the base image.

- Baseline `rawrarr-test-regression-tests-mw0bqhxj`: the Docker healthcheck
  fixture test passes; the desktop signing title assertion reproduces the
  reported failure. Node.js and Rust suites pass.
- Updated `rawrarr-test-regression-tests-mw0bw1a2`: all five Docker healthcheck
  checks and all seven signing topology browser checks pass, including the
  desktop and mobile layout check. Node.js and Rust suites pass. The overall
  run fails on an unrelated 30-second Radarr note timeout in
  `test_rawrarr_coordinator_sonarr_settings.mjs`; that test passed in the
  baseline, and its source is unchanged here.
- `python3 scripts/tests/test_rawrarr_test_runner_fixtures.py`: passes its
  fixture-copy check.
- Confirmation `rawrarr-test-regression-tests-mw0c2iks`: both reported checks
  pass again, as do Node.js and Rust. The same unrelated Radarr note timeout
  fails the browser suite, so the complete runner is not green. No changes
  were made to that test or its application code.

The full multi-node environment integration tests and live deployments are
not exercised by this test-only task.

## Adoption and recovery

Test-only change: no runtime deployment, migration, persistent-state rewrite,
or protocol change. Existing deployments need no action. Reverting the test
change restores the old above-the-fold assumption without affecting runtime.
