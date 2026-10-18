# Summarize failed Rawrarr container tests

## Objective and initial state

Make the final lines in retained test-container logs identify which Node.js or Rust cases failed. The runner printed suite exit codes and `CONTAINER TESTS FAILED`, while the detailed test output appeared earlier in the log.

## Changes

- Captured each suite's output temporarily, then replayed it unchanged into container stdout so existing logs stay readable.
- Added a final per-suite summary framed by a prominent separator, reporting pass/fail and extracting Node TAP failures, Rust failed-test lines, and compiler or runner errors.
- Added coverage to the Docker healthcheck convention test and updated the test index.

## Verification

- `sh -n scripts/tests/run-tests-in-container.sh` passed.
- `node --test scripts/tests/test_rawrarr_docker_healthchecks.mjs` passed (2/2).
- The full isolated Docker run was not repeated; validation covered the updated runner syntax and its regression assertions.

## Final state

The test-container entrypoint ends with suite pass/fail status and extracts failure lines from the retained Node and Rust logs. Existing detailed output and the overall container exit status remain available.
