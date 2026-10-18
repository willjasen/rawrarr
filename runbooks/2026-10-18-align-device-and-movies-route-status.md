# Align device and Movies route status

## Objective

Make the Devices page report direct reachability using the same browser-side health probe used by Movies route selection.

## Initial state

The Devices card labeled a node `Direct` whenever the coordinator advertised a `directUrl`, regardless of whether this browser could reach it. Movies actually probed `/api/direct/health` before selecting direct access, so the labels could disagree.

## Actions

- Reuse the direct health probe for Devices route labels and cache results per node for 15 seconds to avoid repeating checks during the 10-second node polling cycle.
- Label a node Direct only after the browser probe succeeds; otherwise show Relay when the relay is available or Offline.
- Refresh stale assertions in the existing direct-route test for the current session-protected Settings endpoints and latency-bearing Direct status text; add regression coverage for probe-based Devices labels.
- Rebuilt only roarlay in the existing `rawrarr-local` project with the active `docker-compose.yml`, `docker-compose.coordinator.yml`, and `docker-compose.tailscale-macbook.yml` files.

## Verification

- `node --check rust-arr/src/coordinator-workspace.mjs` passed.
- `node --test scripts/tests/test_rawrarr_direct_route.mjs scripts/tests/test_rawrarr_coordinator_workspace.mjs` passed (16 tests).
- `git diff --check` passed for task-owned files.
- Roarlay is healthy; local port 7297 and `https://rawrarr.risk-mermaid.ts.net/health` both returned HTTP 200.
- Confirmed the served coordinator script contains the cached direct health probe and probe-based Devices label.
- Tailscale Serve readback confirms `/` to `http://127.0.0.1:8080` and existing `/direct-api` to `http://127.0.0.1:8082`.
- Removed the task-owned roarlay deployment marker after verification.

## Recovery

Revert the probe cache and `directReachabilityByNode` calculation in `rust-arr/src/coordinator-workspace.mjs` and the corresponding test assertion.
