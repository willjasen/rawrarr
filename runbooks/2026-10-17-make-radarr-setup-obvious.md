# Make Radarr setup easy to find

## Objective and initial state

Make the existing Radarr connection form discoverable from the rawrarr dashboard. The form and API already exist, but the dashboard places the form below other panels without a navigation shortcut. Related context: [Radarr UI configuration](2026-10-17-rawrarr-radarr-ui-config.md).

## Actions

- Add a Settings navigation link and a prominent Configure Radarr action in the connection status card.
- Give the connection panel a stable anchor and scroll it into view from either entry point.

## Verification

- `cargo build --release --locked --manifest-path rust-arr/Cargo.toml` passed with Rust 1.85.1; `git diff --check` passed.
- Restarted the local server on `127.0.0.1:8080`; `curl` confirmed the updated Settings link, Configure Radarr action, and settings-panel anchor are served.
- Reloaded the dashboard in the in-app browser and confirmed the Radarr form, URL field, API-key field, and Save and connect button render. No credentials were entered.

## Final state and remaining work

Radarr setup now has two visible navigation entry points: Settings in the sidebar and Configure Radarr in the connection status card. Both scroll to the form. Connection state and API behavior are unchanged; save still holds credentials in process memory until restart.
