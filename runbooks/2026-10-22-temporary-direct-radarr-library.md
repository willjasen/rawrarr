# Serve the Radarr library through the node direct API

## Objective and initial state

Restore the coordinator Movies view after the encrypted library request returned HTTP 502. History showed two distinct prior failures: a stale coordinator relay hostname, and a later ECDH/HKDF mismatch that returned a decryption error. The current coordinator-to-node and node direct routes were available; Sonarr loaded 229 series. The node returned HTTP 502 for Radarr library reads with `Could not save quality profiles to shared movie metadata`, identifying a node-local cache write failure rather than a relay or API-key validation failure.

The user requested that Radarr use the same direct browser-to-node route as Sonarr for now, with the change recorded for later rollback. Related history: [encrypted library relay 502](2026-10-18-fix-library-relay-502.md), [encrypted response authentication](2026-10-18-fix-encrypted-library-authentication.md), and [direct Sonarr library route](2026-10-18-fix-direct-sonarr-library-404.md).

## Planned changes

- Added a direct-only `/api/radarr/live-library` response that reads Radarr's live inventory and quality-profile names without calling the shared sidecar enrichment/write path. Its DTO omits node-local paths and file details.
- Added `/api/radarr/live-movies/{tmdb_id}/poster`, which reads poster bytes from Radarr/TMDb without persisting them. The coordinator Movies view now uses both endpoints over the same browser-to-node Tailscale direct API as TV and does not retry through the coordinator relay.
- Keep the encrypted Radarr session endpoint available as the rollback path.
- Do not change the saved Radarr URL/API key, Tailscale ACL, Sonarr settings, or shared data.

## Work and verification

- Root cause: the direct `GET /api/radarr/movies` handler and encrypted session both called `ArrClient::list_movies()`, which merges quality-profile metadata into shared encrypted movie sidecars and returned 502 when that write failed. Sonarr uses a live direct response and does not require that Radarr cache write.
- Built `rawrarr:local` from this checkout with Docker; the Rust release build passed. Updated only `fossil` in the existing `rawrarr-node-internal` Compose project. It is healthy and the node's Tailscale `/health` endpoint responds successfully.
- `node --check rust-arr/src/coordinator-workspace.mjs`, the focused Radarr link/direct-route tests (2/2), and `git diff --check` passed. A combined coordinator workspace suite has unrelated existing failures against its checked-in selectors and route-state expectations; details should be added if that suite remains relevant after deployment.
- Deployment of the coordinator workspace update to the Mac mini is still pending.

## Rollback

Restore `unlockMetadata()` in `rust-arr/src/coordinator-workspace.mjs` to call `window.rawrarrMiniExport.loadRadarrLibrary(identity, route.base, ...)` with its existing coordinator relay fallback, and remove the temporary direct-only Radarr library/poster routes and handlers from `rust-arr/src/main.rs`. Rebuild/redeploy the metadata node and internal mini coordinator using their existing Compose projects, then verify the previous encrypted route before removing the direct fallback. The original encrypted implementation remains in `rust-arr/web-export/metadata-export.js` and the existing `POST /api/metadata/session/library` route.

## Verification and remaining work

Record focused checks and live deployment results here. Shared quality-profile sidecar persistence remains a separate issue and should be repaired before restoring the encrypted-library path.
