# Hide topology until a device is online

## Objective

Keep the Devices topology section out of view until at least one enrolled device has successfully reported online. Track implementation and acceptance criteria in [issue #75](https://github.com/willjasen/rawrarr/issues/75).

## Initial state

The section was visible in the no-device empty state, showing a blank network diagram. The workspace already polls enrolled nodes and includes each node's current `online` state. The task starts on `codex/hide-topology-until-online`, based on fetched `origin/main` at `c13edf4`.

## Changes

The topology section is initially hidden in the HTML and is shown only when a returned node has `online === true`. Empty and offline-only responses keep it hidden. The renderer is skipped when there is no online device. Regression and Playwright coverage includes the empty, offline-only, and online states.

## Verification

Passed `node --test scripts/tests/test_rawrarr_coordinator_workspace.mjs` (25/25), `node --check scripts/tests/test_rawrarr_topology_visibility_browser.mjs`, and `git diff --check`. Passed the new Playwright test in `rawrarr-testing-base:local` (3/3), including the 10-second offline-to-online polling transition. The Playwright browser test needs the repository test image because the host does not install its pinned `playwright-core` module.

## Recovery and remaining work

Revert the topology visibility change in `rust-arr/src/coordinator_workspace.html` and `rust-arr/src/coordinator-workspace.mjs`, and remove the task-owned test/index/runbook additions. No deployment is part of this task. The remaining GitHub handoff is to commit and push the branch for PR review.
