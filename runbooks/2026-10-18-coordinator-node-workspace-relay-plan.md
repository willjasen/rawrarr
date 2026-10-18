# Plan the coordinator node workspace and page relay

## Objective

Make `https://rawrarr.risk-mermaid.ts.net/` the signed-in Rawrarr entry point. It should show the existing coordinator and connected metadata/storage nodes, accept pending node enrollment, and display a selected node's Rawrarr pages without requiring users to open `localhost:7297` on each host.

The coordinator is assumed to be already provisioned and healthy. The MacBook metadata node is the seeded participant; the Mac mini storage node is a later joining node. This is a node-connection and workspace change, not coordinator bootstrap.

## Current evidence and why approval is hard to find

- The MacBook metadata dashboard source contains a **Devices** panel and an approval button. The panel polls `/api/nodes/pending` every 10 seconds while the dashboard is open and signed in.
- The metadata container recently returned HTTP 200 for `/api/nodes/pending`, and the current Mini request is still unexpired and associated with the existing owner. The mini process is waiting for approval.
- The Tailscale URL routes to the coordinator, whose root page is currently only its health page. Therefore opening the shared URL cannot show the metadata dashboard's approval control.
- Approval is presently colocated with the metadata dashboard, among its device and integration settings. There is no coordinator-hosted workspace or node-page relay yet.
- Enrollment requests expire after 15 minutes. An expired request disappears from the approval list; restarting the joining node creates a fresh request while preserving its node-local credential.

## Target behavior

1. The user opens `https://rawrarr.risk-mermaid.ts.net/` and signs in there with a passkey. That coordinator-origin session authorizes the workspace and subsequent requests to connected nodes. Keep `/health` as the existing liveness page.
2. The workspace shows coordinator status, metadata/storage nodes, online/last-seen status, and pending enrollment requests. The approval card identifies the node and the owner account it will join. Approval is an explicit action.
3. Selecting a registered node displays that node's Rawrarr workspace inside the coordinator URL. Browser APIs, static assets, and long-lived event streams continue to work through the selected-node route.
4. Metadata and storage nodes both initiate authenticated outbound connections to the coordinator and reconnect safely. Neither node opens a new inbound host port. This is necessary for coordinator-mediated access to the Mac mini behind its own Tailscale container and for a consistent path to the MacBook metadata node.
5. The coordinator routes only to nodes already registered for the signed-in owner's account. A node's ID, hostname, URL, or role supplied by a browser is never sufficient to authorize routing.

## Implementation sequence

### 1. Close the current approval usability gap

Keep the MacBook Devices approval panel working as a fallback while adding an obvious pending-node count and approval card to the coordinator workspace. Show useful states for no request, expired request, coordinator unavailable, and successful approval. Confirm that the owner is signed in to the MacBook metadata UI, that its current build contains the panel, and that the joining node's 15-minute request has not expired. The current UI source contains the panel and its pending API returns HTTP 200; the missing piece is a useful approval surface at the coordinator URL. This avoids conflating that gap with a failed enrollment API.

### 2. Add coordinator-origin passkey sign-in and workspace shell

Add a coordinator-owned passkey login/dashboard page and owner-scoped node list and approval UI. Reuse the existing account record, but provision a passkey credential for the coordinator's HTTPS origin. A credential registered for RP `localhost` cannot authenticate at `rawrarr.risk-mermaid.ts.net`; add an explicit signed-in passkey enrollment step for the coordinator origin rather than silently weakening RP/origin validation or assuming that the existing localhost credential will work. Scope the resulting browser session to the HTTPS origin and keep node credentials out of browser storage.

### 3. Establish node-to-coordinator channels

The metadata node already makes outbound coordinator requests for registration and authentication forwarding, but it has no persistent return channel for coordinator-driven page relay. Have each approved metadata/storage node authenticate using its own node credential and establish an outbound, persistent, reconnecting channel. Include heartbeat, software/role metadata, and bounded relay request/response framing. The coordinator keeps only active connection state in memory; durable authorization remains the approved-node/account registry. Reconnection must restore visibility without creating a second node identity. The metadata and storage processes must not send age identities, Radarr credentials, media files, `/shared` content, or node-local app data over this channel unless a separate explicitly scoped feature requires it.

### 4. Implement an authenticated selected-node relay

Use a coordinator-owned route such as `/nodes/{registered-node-id}/...`. Resolve the node from the signed-in owner's approved registry, then proxy only HTTP methods and paths required by the Rawrarr web app. Preserve request bodies and safe content headers, rewrite redirects and cookie scope deliberately, support bounded responses and SSE, and reject hop-by-hop/proxy headers. Do not create an arbitrary URL proxy or expose node ports publicly. Ensure assets and API calls stay under the selected-node prefix; test relative/absolute paths, refresh/deep links, logout, and simultaneous tabs on different nodes.

Because this self-hosted coordinator terminates browser HTTPS and handles proxied page/API responses, it is a trusted application intermediary for this feature and may see the selected node's UI/API plaintext. It must still have no media mount or private age identity. Keep this distinct from any future hosted blind relay, which must not terminate subscriber TLS or serve relay-controlled page code.

### 5. Switch Tailscale Serve and retain a fallback

Keep the existing Serve route pointed only at the coordinator. Verify its actual route after configuration changes. The coordinator should provide the workspace at `/` and health at `/health`; no broad route to a node listener is needed. Retain `localhost:7297` for local diagnosis until the coordinator-hosted path passes end-to-end tests.

## Decisions and risks to resolve before implementation

- Coordinator WebAuthn RP/origin and explicit passkey enrollment for the Tailscale HTTPS hostname.
- Whether relay authorization uses short-lived signed, node-audience user assertions or a coordinator-validated session on every request; never forward the coordinator's reusable cookie or node bearer token to another node.
- Authenticated channel protocol, framing limits, keepalive/backoff, and how uploads/streams behave across reconnects.
- Safe proxy behavior for SSE, redirects, cookies, CSP, caching, and browser storage under a node path prefix.
- The coordinator is trusted with proxied node pages/API responses, but is still excluded from `/media`, private age IDs, and node-local app data. Radarr credentials must stay local to the node that uses them.
- Whether approval remains a coordinator-native operation exclusively or also stays available in the metadata fallback UI during rollout.

## Acceptance checks

- Anonymous requests cannot list, approve, or open a node workspace. A signed-in owner sees only nodes and enrollment requests assigned to that account.
- A new Mini storage request appears at the coordinator URL with node type, name, expiry, and destination account; explicit approval registers it without changing its node ID or local credential.
- The already seeded MacBook metadata node and a newly approved Mini storage node both establish outbound coordinator channels and appear with accurate online/stale state.
- The coordinator URL can load the selected metadata and storage UI, assets, APIs, and SSE; switching between nodes does not leak cookies, sessions, or state.
- Unregistered IDs, wrong-owner IDs, arbitrary upstream URLs, expired approvals, coordinator restarts, channel drops, and oversized requests fail safely.
- The coordinator has no `/media`, age identity, or node-local app-data mount. Tailnet HTTPS, local `localhost:7297`, and `/health` remain correct.

## Status

Planning only. No application code, Compose configuration, Tailscale Serve state, or live node state was changed for this plan. Implementation should begin with coordinator-origin WebAuthn and the authenticated node-channel design, then add the workspace and relay behind tests before switching the user-facing route.
