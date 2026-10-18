# Show the Radarr movie library in Rawrarr

## Objective and initial state

Expose Radarr's existing movie collection and useful file data in Rawrarr. The configured instance at `127.0.0.1:8080` reports Radarr 6.4.4 as connected; the current encrypted-config build at `127.0.0.1:8081` is not configured. The API key remains in the older process memory and is not available to this implementation. Related references: [encrypted config](2026-10-17-rawrarr-age-encrypted-config.md) and [dashboard UI](2026-10-17-radarr-dashboard-ui.md).

## Actions

- Added a read-only `GET /api/radarr/movies` endpoint that requests the Radarr library and returns selected fields only: title, year, monitored/file state, disk size, path, quality profile, added date, and TMDb ID.
- Added a Movies view with aggregate movie count, file count, disk usage, and a refreshable table.
- The currently configured server was left running. The new build will need its Radarr settings entered in the new instance to retrieve live data.

## Verification

- Rust 1.85 `cargo fmt`, locked `cargo check`, locked release build, and `git diff --check` passed.
- A local Radarr-shaped HTTP fixture confirmed `/api/radarr/movies` returns only the selected library fields and that the API key is not part of the response. The unconfigured server returned the expected `503`.
- Opened the current build on `127.0.0.1:8082` and verified the Movies navigation, summary cards, and table render. The signed-in Radarr page showed 39 movies, all monitored, with 0 movie files and 0 B reported.
- The `8080` Rawrarr instance remains configured and connected; the `8081` instance and current build are not configured. No Radarr key was copied or exposed.

## Final state and remaining work

The read-only movie endpoint and UI are implemented. The current build is running on `127.0.0.1:8082`; configure its Radarr connection to display the live list. Keep `127.0.0.1:8080` running to preserve its in-memory connection until the user migrates the key.
