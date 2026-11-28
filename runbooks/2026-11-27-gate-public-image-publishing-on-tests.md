# Gate public image publishing on tests

## Objective
Ensure public GHCR image tags are built and published only after the Tests workflow passes for the same source commit. GitHub issue [#536](https://github.com/willjasen/rawrarr/issues/536).

## Initial state
The `Tests` and `Publish public container image` workflows both started on pushes to `main`. The publish workflow could push architecture-specific tags while tests were still running; its manifest job waited only for the architecture builds.

## Change
The publish workflow now waits for successful completion of `Tests`. It checks out the commit SHA from the triggering push and uses that SHA for architecture and manifest tags. The architecture jobs and manifest job remain dependent as before. No deployment or persistent data format changes are involved.

## Verification
- Focused workflow tests: `python3 -m unittest scripts.tests.test_publish_container_workflow -v` — passed (5 tests).
- Script suite: `python3 -m unittest discover -s scripts/tests -p 'test_*.py' -v` — passed (157 tests, 5 skipped).
- `git diff --check` — passed.

## Recovery
Revert the workflow change to restore independent push-triggered publication. No runtime migration is required.
