# Restore the current MacBook fossil route

## Objective and tracking

Restore direct and coordinator-relay routing for the MacBook Pro fossil shown as “Online · Route unavailable”. Tracked in [issue #181](https://github.com/willjasen/rawrarr/issues/181).

## Cause

- The MacBook fossil container is healthy and currently reports node ID `device-398f0ce833b6d7e2b9e8f760`.
- The coordinator's published `RAWARR_NODE_RELAY_TARGETS` and `RAWARR_NODE_DIRECT_TARGETS` maps do not contain that current ID, although they retain entries for older MacBook enrollments.
- The verified MacBook tailnet endpoint `https://rawrarr-macbook-pro.risk-mermaid.ts.net/direct-api/health` returns HTTP 200 with `status: ok`. Its response and OPTIONS preflight include the coordinator origin `https://rawrarr.risk-mermaid.ts.net` in the CORS headers.

## Changes and verification

- Added the current MacBook ID to both coordinator route maps, preserving existing entries for older enrollments and the Mac mini.
- Added a focused regression assertion that requires the current ID to appear in both maps with the verified MacBook endpoints.
- `node --test scripts/tests/test_rawrarr_coordinator_workspace.mjs` passed 28/28.
- `docker compose -f docker-compose.internal-mini.yml config -q` passed with placeholder values for protected-file paths; no protected files were read or changed.
- `git diff --check` passed.

## Deployment and live verification

Pending merge and deployment from the resulting published `main` commit. Update only `roarlay` in `rawrarr-internal-mini`; preserve PostgreSQL, Tailscale state, configuration, and browser windows. Confirm its health and coordinator Serve route, then verify the MacBook's current ID is present in both live maps and the authenticated Devices card shows a usable Direct or Relay route.
