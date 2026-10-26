# Add a Devices page entry point for node Compose

## Objective and initial state

Make Docker Compose setup easy to find from Devices after enrollment, so an owner can add another subscriber node. The Devices page only showed the Compose link in its zero-node empty state; the coordinator workspace home already linked to `/setup/node`. Tracked by [issue #34](https://github.com/willjasen/rawrarr/issues/34).

Relevant earlier work: [simplify first-account onboarding](2026-10-22-simplify-first-account-onboarding.md) and [defer subscriber Compose deployment work](2026-10-25-defer-subscriber-compose.md).

## Actions

- Added a persistent Devices page action that opens the existing subscriber setup and Compose generator.
- Updated the focused workspace regression test and runbook index.

## Verification

`node --test scripts/tests/test_rawrarr_coordinator_workspace.mjs` passed all 27 tests. `git diff --check` passed. No live coordinator deployment was requested or performed.

## Recovery and remaining work

Revert the task-owned changes to remove the entry point. The underlying Compose installer still has the deployment gaps tracked in issue #34.
