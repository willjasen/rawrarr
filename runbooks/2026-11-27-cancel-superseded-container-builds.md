# Cancel superseded public container builds

## Objective

Allow the newest public container image publish run to proceed without waiting for an older queued or running publish run. Tracked in [issue #528](https://github.com/willjasen/rawrarr/issues/528).

## Initial state

- `.github/workflows/publish-container.yml` used the stable `publish-container-main` concurrency group with `cancel-in-progress: false`.
- The publish workflow builds AMD64 and ARM64 images and publishes manifest tags after both builds complete.
- Existing focused coverage asserted cancellation was disabled in `scripts/tests/test_publish_container_workflow.py`.

## Changes

- Set `cancel-in-progress: true` while keeping all main publish runs in the same concurrency group. A newer workflow run supersedes older queued or in-progress runs; only the latest successful run publishes the consumer manifest tags.
- Updated the focused workflow test and its index entry.
- No persistent data, schema, configuration, or protocol migration is needed. Existing deployments consume published images; this only changes how future GitHub Actions runs are scheduled.

## Verification

- `python3 scripts/tests/test_publish_container_workflow.py` passed (4 tests).
- `git diff --check` passed.
- Reviewed the workflow change and confirmed the stable `publish-container-main` group remains in place with cancellation enabled.
- A hosted Actions run was not triggered by this local change, so cancellation behavior was not exercised against an active GitHub run.

## Final state

- Future publish workflow runs cancel older queued or in-progress runs in the same group; the newest run can build and publish after both architecture builds succeed.
- Implementation and focused coverage are committed locally as `63caa01` (`fix: cancel superseded image builds (#528)`).
- The branch has not been pushed; hosted Actions behavior remains to be confirmed after review and merge.
