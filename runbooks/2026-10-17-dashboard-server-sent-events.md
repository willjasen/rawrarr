# Push dashboard updates with server-sent events

## Objective and initial state

Replace browser-side 30-second Radarr status polling with a server-sent events stream that pushes dashboard data. Initial state: the dashboard separately loaded Radarr, metadata, library, and device data; Radarr status refreshed on a 30-second browser timer. Related context: [dashboard UI](2026-10-17-radarr-dashboard-ui.md) and [Radarr configuration](2026-10-17-rawrarr-radarr-ui-config.md).

## Actions

- Added `GET /api/events`, an SSE route that emits an initial full dashboard snapshot and then refreshes it every 30 seconds. The snapshot includes Radarr connection state, metadata readiness, library configuration, and scanner devices. Axum SSE keep-alives and the browser EventSource reconnection behavior keep the stream resilient.
- Replaced browser polling language with a live-connection indicator; incoming snapshots update dashboard data without follow-up polling requests. Kept the manual Radarr refresh action.
- Documented the event route and payload cadence in the Rust app README.

## Verification

- Rust 1.85.1 release build passed, followed by `cargo build --release --locked`; `cargo fmt --check`, `git diff --check`, and `docker compose config` passed.
- Started the updated app at `127.0.0.1:8080`. `curl -N` received `event: dashboard` plus the expected JSON snapshot (Radarr unconfigured, library unconfigured, and one local scanner). The request remained open until the intentional 3-second curl timeout.
- Reloaded the in-app browser and verified the live connection indicator changed to “Live updates connected”; dashboard Radarr, metadata, library, and devices rendered from the stream. No credentials were entered during this verification.

## Final state and remaining work

Automatic dashboard updates now use a server-sent events stream instead of browser-side 30-second polling. Each connected dashboard causes the server to refresh Radarr status on the 30-second cadence. Event history/replay is not provided; new connections receive a fresh snapshot immediately.
