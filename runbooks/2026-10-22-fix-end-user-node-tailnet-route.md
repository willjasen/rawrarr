# Fix end-user node tailnet route

## Objective and initial state

Make the newly approved MacBook Pro node reachable from the internal mini coordinator and the Devices page over the owner's Tailscale tailnet. The coordinator was healthy, and PostgreSQL showed a recent heartbeat for the approved node, but the browser route probe showed Offline. The node's app and direct API listeners were healthy on ports 8080 and 8082, while its Tailscale Serve status was empty. The internal coordinator had no relay or direct target for this node.

The user reported adding the `brandons-devices` tag to address an ACL restriction. Existing deployment guidance is in [the internal mini coordinator runbook](2026-10-22-deploy-internal-mini-coordinator.md), and the previous localhost route diagnosis is in [the subscriber local route runbook](2026-10-21-subscriber-device-name-and-local-route.md).

## Changes

- Added Tailscale Serve on the node's sidecar: HTTPS `/` proxies to app port 8080; `/direct-api` proxies to direct API port 8082. No existing Serve handlers were present.
- Configured the internal coordinator to relay this enrolled node through its tailnet HTTPS hostname and to return the node's HTTPS direct API URL to the browser.
- The user's Tailscale tag/ACL change remains user-owned; no policy repository files were edited.

## Verification

Pending. Verify coordinator-to-node TLS requests, deploy the Compose route configuration to the internal mini, inspect the authenticated Devices view, and record final results.

## Recovery and limits

Remove this node's two entries from `docker-compose.internal-mini.yml` and rebuild only the internal mini `roarlay` service to roll back its coordinator routes. To remove the node-side routes, run `tailscale serve --https=443 off` inside the node Tailscale sidecar only if the user asks to disable them. These routes are scoped to the enrolled node and do not delete node data.
