# Test run timing and old-container cleanup

## Objective and initial state

Show test start, stop, and elapsed time in the final isolated test summary. Periodically remove old test containers, with a one-hour interval between cleanup passes. The MacBook's `rawrarr-testing` group contained 46 stopped test-run containers older than one hour during the requested preview; a running base-image container was present and excluded.

## Changes

- Added start and stop timestamps and elapsed seconds to the container test summary.
- Added cleanup to the Docker runner. It examines only stopped containers with the `rawrarr-testing` project label and a `tests-*` service label, and deletes only containers created over one hour ago.
- The last successful cleanup timestamp is kept outside the repository at `~/Library/Application Support/Rawrarr/.test-container-cleanup-last-run`. Normal runner invocations clean only when that timestamp is at least one hour old; `--cleanup-dry-run` lists candidates without deleting or advancing the timestamp.
- Updated test and runner documentation.

## Verification

- MacBook dry run: `python3 scripts/tests/run_rawrarr_tests_in_docker.py --cleanup-dry-run` reported 46 stopped test containers for removal. No containers were removed. The active `base-test-image-mvdgto87` container was excluded.
- `python3 -m py_compile scripts/tests/run_rawrarr_tests_in_docker.py` passed.
- `sh -n scripts/tests/run-tests-in-container.sh` passed.
- `node --test scripts/tests/test_rawrarr_docker_healthchecks.mjs` passed (2 tests).
- Repeated `--cleanup-dry-run` still reported old stopped candidates; no container was removed.

## Final state

Complete. No cleanup timestamp was written and no containers were removed during verification. The normal cleanup will occur on a future runner invocation when the one-hour interval has elapsed.
