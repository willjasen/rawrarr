# Separate node check-in status from browser route status

## Objective and tracking

Resolve the misleading fossil “Offline” label and make its device avatar use the configured coordinator relay when direct browser access is unavailable. Tracked in [issue #86](https://github.com/willjasen/rawrarr/issues/86).

## Initial state

- The coordinator node API reports heartbeat freshness separately from configured coordinator relay and browser direct routes.
- Devices displayed only the browser route label (“Direct”, “Relay”, or “Offline”), which could be mistaken for coordinator heartbeat state.
- Device avatars obtain their node identity seed through the authenticated coordinator relay and therefore require that relay to be configured and the node to be online.
- Internal read-only checks found roarlay healthy, a configured relay target through the MacBook Tailscale Serve URL, a direct target under `/direct-api/`, and no running internal fossil app container. The MacBook app still needs a valid independently saved instance-root fingerprint and subscriber Compose before it can check in.

## Changes and verification

- Device status now distinguishes coordinator check-in from browser route: online nodes say “Online · Direct” or “Online · Relay”; stale nodes say “Not checking in”; online nodes with no browser path say “Route unavailable”.
- Device-avatar loading remains tied to the coordinator relay because its seed is fetched through that authenticated route.
- Focused checks passed: 29 Node tests across coordinator workspace and direct route suites; JavaScript syntax check; `git diff --check`.
- The isolated OrbStack regression runner passed Node 128/128, Playwright 1/1, and Rust 82/82. Python integration tests were skipped because their disposable infrastructure fixtures are not included in that runner.

## Remaining work

After merge, deploy the coordinator UI and verify the signed-in Devices page. The observed coordinator relay/direct maps are keyed to an older device ID than the new approval request, so update them to the approved ID after confirming its running service URL. Restore the fossil subscriber separately when its independently recorded root fingerprint and MacBook Compose environment are available; no trust pin was guessed and no node service or data was changed here.
