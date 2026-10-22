# Show only device route latency

## Objective

Simplify the Devices route status for directly reachable nodes by showing the measured latency alone, such as `123 ms`, without the `Direct` prefix. Keep Relay and Offline labels when latency is unavailable. GitHub issue: [#27](https://github.com/willjasen/rawrarr/issues/27).

## Initial state

The coordinator workspace rendered a direct status as `Direct · <latency> ms`. Work is isolated in a task worktree based on the latest `origin/main`; unrelated changes in the shared checkout were left untouched.

## Actions

- Changed direct status text to show only the latency label. Route indicator styling and its accessible description still convey Direct connectivity; Relay and Offline remain textual states.
- Added a focused regression assertion.

## Verification

- `node --check rust-arr/src/coordinator-workspace.mjs` passed.
- `node --test --test-name-pattern='default metadata selection measures' scripts/tests/test_rawrarr_coordinator_workspace.mjs` passed (1 targeted test).
- `git diff --check` passed. The full workspace test has unrelated existing assertions that do not match current source; the touched latency assertion passes in isolation.

## Recovery

Restore the direct status label composition in `rust-arr/src/coordinator-workspace.mjs` and remove the corresponding assertion in `scripts/tests/test_rawrarr_coordinator_workspace.mjs`.
