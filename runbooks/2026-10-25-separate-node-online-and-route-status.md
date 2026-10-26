# Separate node check-in status from browser route status

## Objective and tracking

Resolve the misleading fossil “Offline” label and make its device avatar use the configured coordinator relay when direct browser access is unavailable. Tracked in [issue #86](https://github.com/willjasen/rawrarr/issues/86).

## Initial state

- The coordinator node API reports heartbeat freshness separately from configured coordinator relay and browser direct routes.
- Devices displayed only the browser route label (“Direct”, “Relay”, or “Offline”), which could be mistaken for coordinator heartbeat state.
- Device avatars obtain their node identity seed through the authenticated coordinator relay and therefore require that relay to be configured and the node to be online.
- Internal read-only checks found roarlay healthy and no running internal fossil app container. The old relay/direct maps used ID `device-9288eb3b039608d16a9102a5`; the newly approved request shown by the user uses `device-d3ee53b1d561fb349955b199`. Both IDs map to the MacBook Tailscale Serve URL in the coordinator Compose so existing access remains intact. The MacBook app still needs a valid independently saved instance-root fingerprint and subscriber Compose before it can check in.

## Changes and verification

- Device status now distinguishes coordinator check-in from browser route: online nodes say “Online · Direct” or “Online · Relay”; stale nodes say “Not checking in”; online nodes with no browser path say “Route unavailable”.
- Device-avatar loading remains tied to the coordinator relay because its seed is fetched through that authenticated route.
- Added relay and direct target entries for the newly approved fossil ID while retaining the prior node target.
- Focused checks passed: 29 Node tests across coordinator workspace and direct route suites; JavaScript syntax check; `git diff --check`.
- The internal coordinator target-map assertion passed in the workspace suite (27/27), `docker compose ... config --quiet` passed with placeholder protected-file paths, and `git diff --check` passed.
- The isolated OrbStack regression runner passed Node 129/129, Playwright 1/1, and Rust 82/82. Python integration tests were skipped because their disposable infrastructure fixtures are not included in that runner.

## Remaining work

PR #88 merged with the UI correction; the target-map follow-up remains to publish. Deploy the coordinator from the exact merged revision and verify roarlay health, HTTPS Serve routing, and authenticated browser routes. Restore the fossil subscriber separately when its independently recorded root fingerprint and MacBook Compose environment are available; no trust pin was guessed and no node service or data was changed here.
