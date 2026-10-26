# Show only device latency

## Objective

On directly reachable device cards, show only the measured latency, such as `6 ms`, without the `Online ·` prefix. Keep the direct route accessible description and existing text for other states. GitHub issue: [#121](https://github.com/willjasen/rawrarr/issues/121).

## Initial state

The shared checkout was clean on `main` at `b2fdc7a`, matching fetched `origin/main`. A dedicated worktree was created at `/Users/willjasen/.codex/worktrees/device-latency-only` on `codex/device-latency-only`. Related earlier work: [show only device route latency](2026-10-22-show-only-device-route-latency.md).

## Actions

- Updated the Devices card so a direct route displays only its measured latency. The existing accessible label still identifies the active direct Tailscale route; Relay and offline states retain their existing text.

## Verification

- `node --check rust-arr/src/coordinator-workspace.mjs` passed.
- Focused Node tests for device route states and metadata-node selection passed (2 tests).
- `git diff --check` passed.

## Recovery

Restore the `Online ·` prefix in the device card's direct latency label and revert its regression assertion.
