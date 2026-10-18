# Optional direct node connections

## Goal

When a browser can reach a selected Rawrarr node directly, load that node's app traffic over the direct route and reduce the amount of traffic carried by the coordinator. Keep the coordinator as the stable portal for sign-in, node discovery, authorization, and route selection. If direct access is unavailable or fails, continue through the existing coordinator route so cross-network access remains available.

The initial self-hosted Tailscale path is implemented for the MacBook metadata node. Same-LAN detection and the future hosted blind relay remain separate planned work. UniFi Site Manager is a product analogy only; Rawrarr defines its own routing and trust model.

## Current state

- The coordinator workspace lists owner-authorized nodes and maps approved node IDs to trusted HTTP(S) targets with `RAWARR_NODE_RELAY_TARGETS`.
- The coordinator currently proxies selected node content. The selected node's UI/API and coordinator endpoints have different security boundaries; the current self-hosted proxy is trusted to see any plaintext it proxies.
- Some node containers are intentionally isolated from host-published ports. The current Mac mini storage deployment does not expose a browser-reachable direct app route.
- A separate target for the future public portal says prefer a direct peer path and fall back to an opaque relay. That future blind relay is distinct from today's trusted coordinator proxy.
- The coordinator remains necessary for access from networks where no direct browser-to-node route exists.
- The MacBook Tailscale Serve endpoint has a dedicated `/direct-api/` handler to a loopback-only metadata API listener. The handler exposes only the encrypted library response, encrypted config, and ciphertext export endpoints.
- The coordinator advertises a direct route only when explicitly configured and same-origin with the portal. It issues a five-minute HttpOnly, Strict, node-scoped session cookie restricted to that direct path after checking the exact browser Origin and owner/node relationship.
- The workspace probes the Tailscale path automatically, reports whether it is active, and retries through the existing coordinator relay if a direct request fails. The direct listener does not expose the node's general UI or other APIs.

See [device architecture](device-architecture.md), [cloud access architecture](cloud-access-architecture.md), [blind coordinator content](blind-coordinator-content.md), and [node linking](coordinator-linking-plan.md).

## Proposed behavior

1. The browser opens the coordinator portal and authenticates there as it does today.
2. The coordinator returns only nodes the account may access, plus route options and a short-lived, narrowly scoped authorization handoff. A node ID or route URL alone never grants access.
3. The client checks for a usable direct Tailscale route to the selected node. For this phase, same-LAN discovery is out of scope. The browser itself must have Tailscale reachability; a node sidecar alone does not make the user's browser a tailnet member.
4. If the browser can establish an authenticated, certificate-valid session with the node, application requests use that direct origin. The node independently authorizes the user and requested operations.
5. If direct reachability, TLS validation, or authorization handoff fails, the UI uses the existing coordinator proxy. Switching routes must not silently weaken authentication or send credentials to an untrusted origin.
6. The UI automatically selects the direct Tailscale route when its authenticated probe succeeds, otherwise uses the coordinator route. It visibly reports the active path and fallback state without exposing bearer tokens or sensitive network details.

The coordinator continues handling login, discovery, node approvals, route tickets, and fallback traffic. Media bytes remain on storage nodes and must not be proxied by the coordinator. Where browser-visible content is required to remain hidden from the coordinator, use the ciphertext-only browser decryption boundary in [blind coordinator content](blind-coordinator-content.md) on both direct and fallback paths.

## Decisions and prerequisites

The product choices for this plan are:

- **Initial direct network:** Tailscale only. Same-LAN discovery is explicitly deferred and can be added later without changing the route-selection contract. The browser must itself have Tailscale reachability; a Tailscale sidecar on a node does not automatically provide it.
- **Route selection:** choose automatically, prefer the direct Tailscale route when its authenticated probe succeeds, and fall back to the coordinator route when it does not. Show the active route and fallback state in the UI.
- **Scope:** design for both the current self-hosted deployment and the future hosted portal, but implement in separate milestones. First validate direct Tailscale routing in the current self-hosted workspace while preserving its trusted coordinator proxy. Keep the future hosted blind relay as a separate milestone with stronger end-to-end TLS requirements.

Technical prerequisites to resolve before implementation:
- **Browser origin and passkeys:** whether direct navigation changes origin; how WebAuthn RP IDs, passkey login, cookies, CSRF protection, and logout behave across coordinator and node origins. Avoid broadening accepted origins. Prefer a coordinator-issued, short-lived, one-use handoff redeemed by the node over forwarding coordinator cookies or long-lived credentials.
- **Node HTTPS identity:** how each node obtains a stable Tailscale hostname and browser-trusted certificate, including renewal and hostname changes.
- **Direct capability advertisement:** how nodes publish health/reachability endpoints and how the browser probes them without exposing private addresses to unauthorized users.
- **Fallback rules:** which failures trigger fallback, timeout budget, retry behavior, and whether the UI offers a manual coordinator-route override.
- **Current trusted relay versus future blind relay:** this plan initially optimizes the self-hosted workspace. It must not claim the current proxy is blind. Hosted relay behavior remains a separate design and must preserve browser-to-node end-to-end TLS if that privacy promise is made.
- **Allowed request surface:** define API/page paths and WebSocket/SSE behavior for direct requests. Ensure CORS, CSP, cookie scope, and origin checks are explicit; do not use permissive wildcard CORS.
- **Exposure and policy:** determine whether LAN/Tailscale node listeners can be exposed safely, and preserve current container isolation unless a narrowly scoped route is approved.

## Implementation sequence

1. Measure coordinator request/response bytes before and after using the same representative UI workload. Confirm only eligible browser-to-node metadata traffic moves off the coordinator; login, discovery, fallback, and node control traffic still use it.
2. Add direct routes for other nodes only after each node has an independently reviewed Tailscale Serve/backend path and same-origin or explicit CORS/origin design. Do not use arbitrary per-browser URLs.
3. For the hosted-portal milestone, separately define an end-to-end browser-to-node TLS path that remains opaque to the hosted relay. Reuse route selection and fallback behavior only if its identity and authorization model meet the hosted threat model; do not treat the current trusted proxy as a blind relay.

## Acceptance criteria

- The coordinator portal remains the entry point and continues to work across different networks through its existing route.
- On the Tailscale milestone, a browser with tailnet access connects to an authorized node directly and receives a valid TLS identity. Same-LAN-only routing is not part of this phase.
- An unavailable or invalid direct route falls back cleanly without exposing credentials or changing node authorization.
- A node rejects missing, expired, replayed, or wrongly scoped handoffs and independently checks authorization.
- No coordinator cookie, passkey assertion, private age identity, or long-lived node token is copied to an untrusted origin.
- Coordinator and node logs contain no route ticket or application payload secrets.
- Browser network evidence shows eligible app requests bypassing the coordinator; measured coordinator bytes fall for that workload. Direct reachability alone is not counted as success.
- Existing coordinator fallback and local access remain functional.
- The design provides a distinct hosted-portal milestone whose relay cannot terminate browser-to-node TLS or inspect app content.

## Remaining technical decisions

- Decide whether users need a manual coordinator-route override after automatic selection.
- Set explicit probe and fallback timeouts and retry policy.
- Measure coordinator traffic savings for the representative workspace flow.
- Define hosted relay tickets, TLS passthrough, node origin identity, and abuse controls independently from the self-hosted milestone.
