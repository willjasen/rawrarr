# Fix end-user node tailnet route

## Objective and initial state

Make the newly approved MacBook Pro node reachable from the internal mini coordinator and the Devices page over the owner's Tailscale tailnet. The coordinator was healthy, and PostgreSQL showed a recent heartbeat for the approved node, but the browser route probe showed Offline. The node's app and direct API listeners were healthy on ports 8080 and 8082, while its Tailscale Serve status was empty. The internal coordinator had no relay or direct target for this node.

The user reported adding the `brandons-devices` tag to address an ACL restriction. Existing deployment guidance is in [the internal mini coordinator runbook](2026-10-22-deploy-internal-mini-coordinator.md), and the previous localhost route diagnosis is in [the subscriber local route runbook](2026-10-21-subscriber-device-name-and-local-route.md).

## Changes

- Added Tailscale Serve on the node's sidecar: HTTPS `/` proxies to app port 8080; `/direct-api` proxies to direct API port 8082. No existing Serve handlers were present.
- Configured the internal coordinator to relay this enrolled node through its tailnet HTTPS hostname and to return the node's HTTPS direct API URL to the browser.
- The user's Tailscale tag/ACL change remains user-owned; no policy repository files were edited.

## Verification

- The owner added the `brandons-devices` tag to the node. No Tailscale policy file was changed by this task.
- Confirmed the node's Tailscale sidecar and both local app listeners were healthy. Added persistent Serve routes in that sidecar: `/` to `127.0.0.1:8080` and `/direct-api` to `127.0.0.1:8082`.
- After the tag change, coordinator-origin TLS requests to both node routes returned HTTP 200. Before the tag change, TCP requests to the peer timed out.
- Updated `docker-compose.internal-mini.yml` with this enrolled node's relay target and direct API URL. `docker compose config -q` passed on the Mac mini. Recreated only `roarlay`; PostgreSQL stayed healthy.
- After deployment, the mini coordinator returned HTTP 200 from both the node app and direct API over Tailscale HTTPS. Internal and tailnet coordinator health checks returned HTTP 200; roarlay is healthy with zero restarts.
- Verified the coordinator's authenticated relay route from Chrome: `/nodes/device-9288eb3b039608d16a9102a5/api/syncthing/shared-health` returned `{"configured":false,"folders":null}` from the node. This confirms coordinator-to-node API proxying; Syncthing is not configured on this node.
- The authenticated Devices page now shows `Direct · 50 ms` for MacBook pro instead of Offline. This confirms the browser's direct health probe can reach the app through Tailscale Serve.
- Released and confirmed removal of the roarlay deployment marker.

## Recovery and limits

Remove this node's two entries from `docker-compose.internal-mini.yml` and rebuild only the internal mini `roarlay` service to roll back its coordinator routes. To remove the node-side routes, run `tailscale serve --https=443 off` inside the node Tailscale sidecar only if the user asks to disable them. These routes are scoped to the enrolled node and do not delete node data. New internal subscriber nodes will need corresponding Serve and coordinator route entries until onboarding can configure those routes automatically.
