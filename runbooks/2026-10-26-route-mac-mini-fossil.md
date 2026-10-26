# Route the Mac mini fossil through Tailscale Serve

## Objective and tracking

Fix the newly enrolled Mac mini fossil's missing Devices avatar and browser route. Tracked in [issue #136](https://github.com/willjasen/rawrarr/issues/136).

## Initial state

- The Devices card shows a fresh coordinator heartbeat as “Online · Route unavailable”, a broken profile image, and unavailable Syncthing health.
- Coordinator heartbeat freshness and browser-route reachability are separate signals; “Online” indicates the node checked in recently.
- The coordinator's configured route maps currently contain the older MacBook fossil IDs, but not this new node ID.
- Earlier read-only diagnostics found the Mac mini subscriber's Tailscale sidecar had no Serve configuration and roarlay had no relay/direct map entries for the new node.
- Related background: [separate node check-in and browser route status](2026-10-25-separate-node-online-and-route-status.md).

## Actions and verification

- Confirmed the new node ID is `device-86fc42158c1426b89d460f59`; its app listeners respond locally on ports 8080 and 8082, while its Tailscale sidecar initially had no Serve config.
- Added persistent Tailscale Serve handlers in the subscriber sidecar: `/` to `http://127.0.0.1:8080` and `/direct-api/` to `http://127.0.0.1:8082`. Existing Serve config was empty.
- Serve status reads back both HTTPS paths on `rawrarr-mac-mini.risk-mermaid.ts.net`. From the Mac mini, both `/health` and `/direct-api/api/direct/health` returned HTTP 200 when the request was pinned to the subscriber's Tailscale IP; direct health returned `status: ok`.
- Added this node ID to the internal coordinator's relay and direct-target maps and added assertions for both mappings.
- `node --test scripts/tests/test_rawrarr_coordinator_workspace.mjs` passed 27/27; `docker compose -f docker-compose.internal-mini.yml config -q` and `git diff --check` passed.
- PR [#137](https://github.com/willjasen/rawrarr/pull/137) contains the coordinator map change; its required Basic script tests passed. The code mapping is not deployed yet. Until it is published and roarlay is updated, the coordinator will still show no route and will not fetch the avatar or shared-health response through its relay.
- The local workstation could not reach the tailnet hostname directly; testing from the Mac mini host succeeded. The authenticated browser card has not yet been rechecked after coordinator map deployment.

## Final state and remaining work

Subscriber-side Serve is configured and both endpoint paths are healthy over Tailscale. PR #137 is open; merge and published-main deployment must follow the project workflow before end-to-end Devices verification and closing related issue #135.
