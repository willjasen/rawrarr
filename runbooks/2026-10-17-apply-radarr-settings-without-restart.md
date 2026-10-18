# Apply Radarr settings without restarting rawrarr

## Objective and initial state

Make it clear that Radarr credentials entered in the dashboard apply to the running server immediately. Initial implementation already replaces the active Radarr client in memory on `PUT /api/settings/radarr`, but its confirmation said the settings were “active until rawrarr restarts,” which could sound like a restart was required. Related context: [Radarr UI configuration](2026-10-17-rawrarr-radarr-ui-config.md) and [SSE dashboard updates](2026-10-17-dashboard-server-sent-events.md).

## Actions

- Reworded the Radarr form and save confirmation to state that updates apply immediately without a server/container restart. Clarified that the API key is only lost if the rawrarr process itself restarts.
- Corrected the unconfigured status guidance to point to the dashboard form rather than requiring environment configuration.

## Verification

- Rust 1.85.1 release build with `--locked`, `cargo fmt --check`, and `git diff --check` passed.
- Ran a separate rawrarr process on port 18080 and a local Radarr mock on port 18787. Without restarting rawrarr, `PUT /api/settings/radarr` succeeded, `/api/radarr/status` returned the mock Radarr version, and the SSE dashboard snapshot reported configured and connected. The mock used a disposable test credential; no real key was read or printed. Both temporary processes were stopped.
- Restarted the dashboard on port 8080 and verified the updated no-restart wording and live event connection in the browser. Its Radarr configuration remains unset.

## Final state and remaining work

Entering and saving the Radarr URL/API key now clearly applies it to the running process immediately. Rawrarr does not restart. The API key remains memory-only and must be re-entered after an actual process/container restart; persisting it securely is a separate future decision.
