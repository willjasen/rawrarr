# Verify GitHub UI refreshes

## Objective

Add a regression test proving the UI updater reads a GitHub release, fetches its bundle and checksum with the configured token, installs the bundle and release metadata, and recognizes the same release on its next check.

## Environment and initial state

- The updater implementation is in `rust-arr/update_ui_bundle.py`; `scripts/tests/test_ui_bundle.py` already covered checksum and archive validation with mocked fetches.
- A previous deployment discussion exposed a gap: no automated evidence demonstrated the full release-to-installed-node path. Live node access was unavailable during that verification, so this task adds deterministic local regression coverage without claiming a live refresh.
- Existing worktree has unrelated staged, modified, deleted, and untracked files. They are preserved and excluded from this task.

## Changes

- Extend the UI bundle tests with a mocked GitHub release API flow. The test asserts the latest-release endpoint, both asset endpoints, bearer token propagation (using a fixture token), binary asset accept headers, extracted UI content, release commit metadata, checksum metadata, and the second-check `current` result.
- Update the test index and this runbook index entry.

## Verification

- Run `python3 scripts/tests/test_ui_bundle.py -v` and the repository's standard-library suite with `python3 -m unittest discover -s scripts/tests -p 'test_*.py' -v`.
- This test uses mocked GitHub responses. It proves updater behavior and protects against regressions, but does not by itself prove any live node fetched a release; live evidence requires reading that node's installed bundle metadata or updater output.

## Recovery and remaining work

- The test does not alter services or protected files; revert only its task-owned test, test-index, runbook, and runbook-index changes if needed.
- No live node refresh is performed as part of this test-only task.
