# Configure Radarr from the rawrarr UI

## Objective and initial state

Let the user configure Radarr from the local rawrarr dashboard while preserving the optional environment-based setup. Initial state: Radarr URL and API key are read only at process startup; the dashboard shows instructions but has no form. The service is bound to localhost in development and Docker publishes only on host loopback. Related context: [Radarr integration](2026-10-17-radarr-first-integration.md), [metadata matching](2026-10-17-movie-metadata-matching.md), and [local scan](2026-10-17-local-library-scan.md).

## Actions

- Added a dashboard form for the Radarr base URL and API key, plus clear/status feedback. URL loading never returns the stored key to the page.
- Added `GET`, `PUT`, and `DELETE /api/settings/radarr`. The API validates HTTP(S) URLs and API key header values, keeps the credential in process memory, and reuses the connection for health and metadata requests.
- Kept `RADARR_URL` and `RADARR_API_KEY` startup configuration for Docker and headless deployments. UI-entered settings are intentionally lost when the app restarts so raw credentials are not written to disk.

## Verification

- `cargo fmt`, release build, locked release build, Compose config, and `git diff --check` passed.
- Local HTTP mock exercised invalid URL rejection, configuration save/read without key disclosure, Radarr status, movie lookup, and clearing the active configuration.
- Refreshed the dashboard at `http://127.0.0.1:8080/` and verified the URL/key fields and save/clear buttons render. The local process remains unconfigured; no user key was entered.

## Final state and remaining work

Radarr can be configured from the local dashboard or startup environment. A UI-entered API key lasts only until restart. The unauthenticated dashboard remains loopback-only; add access control before exposing it to a LAN or tailnet. Persistent app-owned library management remains the next core milestone.
