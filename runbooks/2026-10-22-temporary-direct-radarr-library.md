# Serve the Radarr library through the node direct API

## Objective and initial state

Restore the coordinator Movies view after the encrypted library request returned HTTP 502. History showed two distinct prior failures: a stale coordinator relay hostname, and a later ECDH/HKDF mismatch that returned a decryption error. The current coordinator-to-node and node direct routes were available; Sonarr loaded 229 series. The node returned HTTP 502 for Radarr library reads with `Could not save quality profiles to shared movie metadata`, identifying a node-local cache write failure rather than a relay or API-key validation failure.

The user requested that Radarr use the same direct browser-to-node route as Sonarr for now, with the change recorded for later rollback. Related history: [encrypted library relay 502](2026-10-18-fix-library-relay-502.md), [encrypted response authentication](2026-10-18-fix-encrypted-library-authentication.md), and [direct Sonarr library route](2026-10-18-fix-direct-sonarr-library-404.md).

## Planned changes

- Added a direct-only `/api/radarr/live-library` response that reads Radarr's live inventory and quality-profile names without calling the shared sidecar enrichment/write path. Its DTO omits node-local paths and file details.
- Added `/api/radarr/live-movies/{tmdb_id}/poster`, which reads poster bytes from Radarr/TMDb without persisting them. The coordinator Movies view now uses both endpoints over the same browser-to-node Tailscale direct API as TV and does not retry through the coordinator relay. Poster images use direct URLs, matching the TV view.
- Keep the encrypted Radarr session endpoint available as the rollback path.
- Do not change the saved Radarr URL/API key, Tailscale ACL, Sonarr settings, or shared data.

## Work and verification

- Root cause: the direct `GET /api/radarr/movies` handler and encrypted session both called `ArrClient::list_movies()`, which merges quality-profile metadata into shared encrypted movie sidecars and returned 502 when that write failed. Sonarr uses a live direct response and does not require that Radarr cache write.
- Built `rawrarr:local` from this checkout with Docker; the Rust release build passed. Updated only `fossil` in the existing `rawrarr-node-internal` Compose project. The final fossil image is `sha256:cbf336ab5d93c6a448fa186098276ae13c1edf0ad780214087856d8faea4db8f`, it is healthy with zero restarts, and its Tailscale `/health` responds HTTP 200.
- Backed up the mini coordinator's prior workspace source to `~/Library/Application Support/rawrarr/internal-coordinator/backups/pre-1fc7a1d/coordinator-workspace.mjs`, then later saved the first deployed version to `backups/pre-poster-fix-1fc7a1d/`. Updated only `roarlay` in Compose project `rawrarr-internal-mini`; PostgreSQL and the Tailscale sidecar were not restarted. The final image is `sha256:697cc7a6831eb3f0df8e79006e7ff4693936436a95add57479c18719dd30817c`, healthy with zero restarts.
- The first remote build attempt lacked `/usr/local/bin` in noninteractive SSH `PATH`, so Docker could not find its configured credential helper. Retried with that directory in `PATH`; the build completed. Remote commands ran without tmux because the Mac mini has no tmux.
- Live authenticated Chrome verified the Movies view loaded 1,945 entries, 1,906 with files, and about 8.0 TB. The displayed quality profiles and row values rendered; the direct poster endpoint returned an image. `https://rawrarr.risk-mermaid.ts.net/health` and mini loopback health returned HTTP 200; Tailscale Serve still maps HTTPS `/` to `http://127.0.0.1:8080`. The deployed workspace script's SHA-256 matched the committed source.
- `node --check rust-arr/src/coordinator-workspace.mjs`, the focused Radarr link/direct-route tests (2/2), the Docker Rust release build, and `git diff --check` passed. A broader existing coordinator-workspace assertion suite reported seven failures against current page selectors and route-state expectations; the focused direct Radarr assertions passed.
- Both deployment locks were released after health, route, and browser verification.

## Rollback

Revert this task's source commits, then rebuild/redeploy only `fossil` in `rawrarr-node-internal` and only `roarlay` in `rawrarr-internal-mini` using the existing projects and their per-container deployment locks. The mini's pre-task workspace asset is preserved at `~/Library/Application Support/rawrarr/internal-coordinator/backups/pre-1fc7a1d/coordinator-workspace.mjs`. Verify Movies through the previous encrypted session path after rollback. The original encrypted implementation remains in `rust-arr/web-export/metadata-export.js` and the existing `POST /api/metadata/session/library` route.

## Verification and remaining work

Record focused checks and live deployment results here. Shared quality-profile sidecar persistence remains a separate issue and should be repaired before restoring the encrypted-library path.
