# Prefer direct metadata nodes in roarlay

## Objective

When the roarlay workspace chooses a default metadata node, measure online direct routes and choose the lowest-latency reachable one, while preserving explicit node selection and relay fallback.

## Initial state

- The coordinator workspace selected the first online metadata node in the coordinator response order.
- Direct route probing and relay fallback already existed for the selected node.
- The checkout contains extensive unrelated user changes; only this runbook, its index entry, the workspace script, and its focused test are task-owned.

## Actions

- For initial default selection, probe all online metadata nodes with a configured direct route concurrently, using the existing 2.5-second timeout, and choose the successful probe with the lowest measured browser-to-node latency.
- Keep explicit requested or saved selection authoritative. If no direct probe succeeds, use the first online relay-capable metadata node, as before.
- Reuse the winning probe result to establish its node-scoped direct session; fall back to that node's roarlay relay if session issuance fails.
- Show the selected direct probe latency in the route status.
- Keep the encrypted-config source's existing selection order unchanged; these cross-origin direct routes are scoped to the protected metadata APIs.
- Added focused regression assertions for concurrent probing, latency ordering, and relay fallback.

## Verification

- `node --test scripts/tests/test_rawrarr_coordinator_workspace.mjs`: all 8 tests passed. Updated a stale selection assertion to match the block form preserving explicit requested-node selection.
- `git diff --check` for task paths: passed.
- Deployment was not attempted because coordinator source and Compose configuration have extensive pre-existing uncommitted changes. The live coordinator has not been verified against this source.
- Changes remain local and uncommitted. The workspace script and focused test are untracked parts of the existing coordinator implementation; the shared runbook index also contains extensive unrelated edits, so this task did not stage or commit any files.

## Final state

- The first default metadata choice now uses the lowest-latency successful direct probe among online metadata nodes, while explicit saved/requested selection and relay fallback remain intact. Live latency and deployment behavior were not verified.
