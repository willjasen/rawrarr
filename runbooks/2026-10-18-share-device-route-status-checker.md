# Share the device route status checker

## Objective

Provide one reusable coordinator helper that checks every device's browser-visible Direct, Relay, or Offline route and use it on Devices, Movies, and TV.

## Initial state

Devices and Movies had separate route-status logic. The direct probe was embedded in the workspace script and Devices calculated its own reachability map, making future pages likely to duplicate route semantics.

## Actions

- Extract `checkDeviceRoutes` into `rust-arr/src/coordinator-device-routes.mjs`; return a map keyed by node ID with `status` and an optional usable `directRoute`.
- Cache successful and failed direct probes for 15 seconds across all callers, with explicit refresh support.
- Serve the shared module as `/assets/coordinator-device-routes.mjs`.
- Use the same status map for Devices labels, automatic direct-node selection, and Movies/TV route selection.
- Add the shared route-status indicator to TV and refresh it through the same route selector when opening that page.
- Add unit and static regression coverage and update test indexes.
- Rebuilt only roarlay in the existing `rawrarr-local` project with the active `docker-compose.yml`, `docker-compose.coordinator.yml`, and `docker-compose.tailscale-macbook.yml` files.

## Verification

- `node --check rust-arr/src/coordinator-workspace.mjs` and `node --check rust-arr/src/coordinator-device-routes.mjs` passed.
- Four focused Node suites passed (22 tests): device-route helper, coordinator workspace, direct route, and device names.
- Scoped `git diff --check` passed.
- The Chromium Sonarr settings test could not run in this shell because `/usr/local/lib/node_modules/playwright-core/index.mjs` is not installed here.
- Roarlay is healthy; local and tailnet health endpoints returned HTTP 200.
- Confirmed both shared JS assets are served over HTTPS with the common status function and TV status handling.
- Tailscale Serve readback confirms `/` to `http://127.0.0.1:8080` and existing `/direct-api` to `http://127.0.0.1:8082`.
- Removed the task-owned roarlay deployment marker after verification.

## Recovery

Restore the embedded direct probe in `coordinator-workspace.mjs`, remove the shared module asset route, and revert the helper tests and test-index entries.
