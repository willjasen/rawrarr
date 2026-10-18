# Radarr dashboard UI

## Objective and initial state

Replace the plain-text root experience with an actual browser UI for the Radarr-focused Rust app. The local server returned `ok` only from `/health`; the Radarr status API existed. Relevant implementation history: [the Radarr integration runbook](2026-10-17-radarr-first-integration.md) and [local server runbook](2026-10-17-run-rust-arr-locally.md).

## Actions

- Added a responsive dark dashboard at `/` with workspace navigation, Radarr connection state, setup instructions, empty movie-library state, and a clear note that movie inventory and remote agents are future work.
- The page polls the existing Radarr status API at startup, every 30 seconds, and on manual refresh. API key values remain server-side.
- Embedded the UI from `src/ui.html` using `include_str!`; no frontend dependencies or unsafe Rust were added.
- Updated the Rust Arr README with the dashboard URL and routes.

## Verification

- `cargo fmt` and `cargo build --release --locked` passed using Rust 1.85.1.
- Opened `http://127.0.0.1:8080/` in the in-app browser and verified the rendered dashboard and its `Not configured` Radarr state.
- `GET /health` returned `ok`; `/api/radarr/status` returned the expected `503` because Radarr is not configured.
- No live Radarr calls or library changes were made.

## Final state and remaining work

The local server now presents an actual dashboard at `http://127.0.0.1:8080/` while remaining bound to loopback in this development session. Movie inventory and other management views remain future work.
