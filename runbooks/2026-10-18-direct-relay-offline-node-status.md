# Show direct, relay, or offline node status

## Objective and initial state

Coordinator node buttons were labeled Online or Offline, and clicks were ignored for nodes without a coordinator relay target. That prevented direct-only nodes from opening even when a Tailscale route was configured.

## Actions

- Allow node selection when either a coordinator relay or direct Tailscale route is configured.
- Show Direct for recently online nodes with a direct route, Relay for recently online nodes with only a coordinator route, and Offline when the heartbeat is stale or no route is configured.
- Keep route availability details accurate and announce the status in the button's accessible label.
- Extend focused source assertions for route-based selection and status rendering.

## Verification

- `node --test scripts/tests/test_rawrarr_coordinator_workspace.mjs`: 6 passed.
- `git diff --check`: passed.
- Deployment was not attempted because the coordinator source and Compose topology have extensive pre-existing uncommitted user changes; safely applying this UI change would require rebuilding that coordinator.

## Final state and remaining work

- Source and focused assertions are updated. The live coordinator has not been rebuilt, so the requested behavior is not yet visible at the Tailnet URL.
- This task did not stage or commit changes because the affected application files are part of the existing untracked workspace implementation and cannot be isolated from it safely.
