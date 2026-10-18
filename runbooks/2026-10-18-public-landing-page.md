# Add the public Rawrarr landing page

## Objective and initial state

Give signed-out visitors a public introduction to Rawrarr before passkey sign-in. The coordinator root route previously redirected unauthenticated visitors directly to `/auth`; authenticated visitors used `/` for the private workspace. The UI target is roarlay under the project deployment instructions. Relevant product guidance: [design guide](../docs/DESIGN.md), [Rawrarr security](../docs/SECURITY.md), and [product direction](../docs/product-direction.md).

## Changes

- Add a responsive public landing page at `/` for signed-out coordinator visitors; keep authenticated workspace routing and the `/auth` passkey page.
- Describe current Radarr/Sonarr and node capabilities, media placement, and privacy goals. Disclose coordinator-visible operational metadata, incomplete content flows, and the lack of an independent security audit.
- Add a Chromium browser regression test for signed-out route wiring, sign-in links, maturity/security copy, and narrow viewport layout.

## Verification and deployment

- Focused Playwright test `test_rawrarr_landing_page.mjs`: passed in `rawrarr-testing-base:local` (Chromium).
- `git diff --check`: passed for task files and indexes.
- The first deployment exposed that the active coordinator uses `coordinator::router()`, not the generic app's dashboard handler. Updated `coordinator_home` and redeployed only `roarlay`.
- Rebuilt the existing `rawrarr-local` roarlay service with the protected PostgreSQL password-file path. Container state is running/healthy; local and tailnet `/health` identify the responding roarlay service.
- Verified `/` serves the landing page over both `http://127.0.0.1:7297/` and `https://rawrarr.risk-mermaid.ts.net/`, and `/auth` still serves the passkey sign-in page.
- Read back Tailscale Serve JSON: `rawrarr.risk-mermaid.ts.net:443` `/` still proxies to `http://127.0.0.1:8080`; existing `/direct-api` and Syncthing TCP 22000 handlers remain present.
- Acquired and released `.container-rebuild-roarlay.lock`; it is confirmed absent after verification.

## Final state

The public landing page is deployed on roarlay. Signed-out visitors see the landing page at `/`; passkey sign-in remains at `/auth`; authenticated visitors retain the coordinator workspace at `/`. No other containers or volumes were changed.
