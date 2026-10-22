# Fix subscriber coordinator origin validation

## Objective and starting state

The internal subscriber setup rejected the Mac mini coordinator URL with “The coordinator must have an HTTPS origin.” The UI accepted a plain origin only when its text exactly matched `URL.origin`, so a trailing root slash was rejected. The coordinator is served by `rawrarr-internal-mini` on the Mac mini at `https://rawrarr.risk-mermaid.ts.net/`.

## Changes

- Normalize HTTPS coordinator and Headscale control-server URLs to their parsed origins. Accept a trailing root slash, while rejecting credentials, non-root paths, queries, and fragments.
- Pre-fill the internal coordinator field with the deployed Mac mini URL.
- Rebuild the embedded subscriber onboarding browser bundle.

## Verification and deployment

- The focused Node smoke check accepted `https://rawrarr.risk-mermaid.ts.net/` and confirmed Compose contains the normalized URL without the trailing slash.
- Built `subscriber-onboarding.js` with `npm run build:subscriber-onboarding`.
- Deployed the updated onboarding source and bundle to `rawrarr-internal-mini` on the Mac mini. Roarlay returned to healthy with zero restarts; `https://rawrarr.risk-mermaid.ts.net/health` returned HTTP 200 and the Tailscale Serve JSON still pointed HTTPS 443 to `http://127.0.0.1:8080`. The browser bundle returned HTTP 200. The authenticated setup page returned HTTP 303 to an unauthenticated curl request; in the already-open authenticated setup page, setting the coordinator URL and preparing Compose produced a visible preview with the normalized coordinator URL and the usual local-secret-file message. The page was not reloaded, preserving its in-memory setup data.

## Recovery

If the new image does not become healthy or the Tailscale Serve route fails, restore the prior source files from Git and rebuild only `rawrarr-internal-mini-roarlay-1`. Keep its PostgreSQL and Tailscale state volumes; do not stop the mini's subscriber project or alter native Tailscale Serve handlers.
