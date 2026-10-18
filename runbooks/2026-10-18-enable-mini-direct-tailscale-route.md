# Enable the Mac mini direct Tailscale route

## Objective and initial state

Add a browser-usable direct route to the Mac mini storage node, matching the MacBook metadata node. The Mini was online through its existing relay and had a distinct Tailscale HTTPS origin, `rawrarr-mini.risk-mermaid.ts.net`. Its current Serve root route targeted the storage proxy, but it had no dedicated direct API listener or `/direct-api` handler. The coordinator direct-session flow only allowed same-origin metadata nodes, so a health-only route would have misreported direct availability without supporting authenticated requests.

Related history: [Deploy the Mac mini storage node](2026-10-18-mac-mini-storage-node-deployment.md), [Restore the Mac mini node relay route](2026-10-18-fix-mini-node-relay.md), [Implement the Tailscale direct node route](2026-10-18-implement-tailscale-direct-node-route.md), and [Reconnect the MacBook metadata namespace](2026-10-18-reconnect-macbook-metadata-namespace.md).

## Actions

- Extend configured direct targets to permit HTTPS paths on the tailnet's `risk-mermaid.ts.net` names, and allow owner-authorized storage nodes to receive short-lived direct sessions.
- Return the node-scoped session token without caching; on a cross-origin direct node, exchange it for a host-only, HttpOnly cookie through an exact-origin CORS policy. Keep the existing same-origin MacBook cookie flow.
- Allow storage nodes to run the existing bounded direct API on a separate loopback listener. Configure the Mini's storage container and Tailscale Serve `/direct-api` handler while preserving its existing root route and data mounts.
- Advertise the Mini's direct URL from roarlay while retaining its relay target as fallback.

## Verification

- `node --check` passed for the coordinator workspace and shared route checker; three focused Node suites passed (18/18), including direct target, route selection, fallback, and UI checks. Scoped `git diff --check` passed.
- The Mini's Rust service compiled successfully as a native ARM64 image. Recreated only its storage container; its `/data`, `/shared`, and read-only `/media` mounts remained intact.
- Mini local app health returned HTTP 200 and its loopback direct health returned HTTP 204. Mini Tailscale Serve readback preserves `/` -> `http://127.0.0.1:8082` and adds `/direct-api` -> `http://127.0.0.1:8083`.
- The public Mini root health returned HTTP 200 and `/direct-api/api/direct/health` returned HTTP 204. Exact-origin CORS preflight and direct-session cookie bootstrap returned HTTP 204 with the expected headers and `Path=/direct-api/`.
- Rebuilt roarlay in the existing `rawrarr-local` Compose project. It reports healthy; local and public coordinator health return HTTP 200, and the MacBook Serve root and metadata `/direct-api` route remain unchanged.
- Refreshed the signed-in Devices page: both MacBook Pro and Mac mini show **Direct**. The Mac mini coordinator relay target remains configured as fallback.
- Removed the task-owned Mini storage, Mini Tailscale Serve, and MacBook roarlay deployment markers after verification.

## Recovery and remaining work

Remove only the Mini `/direct-api` Serve handler and revert its storage direct-listener settings if the route fails. Preserve the Mini root Serve handler, Tailscale identity, Rawrarr `/data`, shared folder, Syncthing state, and read-only media mount.
