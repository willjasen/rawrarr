# Generate a dinosaur profile picture from the public age ID

## Objective and initial state

Replace the geometric profile placeholder with a distinctive generated dinosaur, while making the result deterministic from the shared public age recipient. The existing Settings profile already showed the account display name and generated its geometric picture in the browser. The coordinator workspace is served by `roarlay`; the public recipient is available from an authorized metadata node. The checkout contains extensive pre-existing user changes, which were preserved.

## Actions

- Rendered an SVG dinosaur in the browser using a SHA-256 digest of the public age recipient as its deterministic feature stream. Dinosaur silhouette, palette, crest, spikes, and markings vary by identity.
- Read the public recipient from an authorized metadata-node endpoint through the existing coordinator relay. The coordinator does not persist it. The picture is generated client-side, with no image or avatar seed stored in PostgreSQL.
- Added profile status copy, focused regression coverage, and the product decision to the design guide.

## Verification

- `node --test scripts/tests/test_rawrarr_coordinator_workspace.mjs`: 10 passed. `node --check rust-arr/src/coordinator-workspace.mjs` and `git diff --check` passed.
- Confirmed local port 7297 was unoccupied and port 7298 belonged to OrbStack's existing Tailscale namespace. The existing `rawrarr-local` Compose project was rebuilt with its coordinator and Tailscale overlays; only `roarlay` was recreated.
- `rawrarr-local-roarlay-1` reports healthy. Local `http://127.0.0.1:7298/health` and tailnet `https://rawrarr.risk-mermaid.ts.net/health` both returned HTTP 200. Tailscale Serve read-back confirms `/` still proxies to `http://127.0.0.1:8080` and `/direct-api` remains configured.
- The deployed workspace module contains the public-recipient fetch and local dinosaur SVG generator. The authenticated profile rendering was not exercised in a signed-in browser session.

## Final state and remaining work

- The signed-in Settings profile now renders a deterministic dinosaur variation derived from the SHA-256 digest of the shared public age recipient. No image, recipient, or avatar seed is written to PostgreSQL. The deployed Settings view is available at the coordinator URL; live user-specific rendering awaits an authenticated browser session.
