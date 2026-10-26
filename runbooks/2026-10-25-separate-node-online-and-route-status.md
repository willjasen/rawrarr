# Separate node check-in status from browser route status

## Objective and tracking

Resolve the misleading fossil “Offline” label and make its device avatar use the configured coordinator relay when direct browser access is unavailable. Tracked in [issue #86](https://github.com/willjasen/rawrarr/issues/86).

## Initial state

- The coordinator node API reports heartbeat freshness separately from configured coordinator relay and browser direct routes.
- Devices displayed only the browser route label (“Direct”, “Relay”, or “Offline”), which could be mistaken for coordinator heartbeat state.
- Device avatars obtain their node identity seed through the authenticated coordinator relay and therefore require that relay to be configured and the node to be online.
- The initial Docker inventory was taken on the Mac mini and did not include MacBook containers; it was not evidence that the MacBook app was stopped. The old coordinator relay/direct maps used ID `device-9288eb3b039608d16a9102a5`, while the newly approved node uses `device-d3ee53b1d561fb349955b199`. The approved node was checking in, but its new ID had no matching route entry, so the browser could neither relay to it nor load its device avatar.

## Changes and verification

- Device status now distinguishes coordinator check-in from browser route: online nodes say “Online · Direct” or “Online · Relay”; stale nodes say “Not checking in”; online nodes with no browser path say “Route unavailable”.
- Device-avatar loading remains tied to the coordinator relay because its seed is fetched through that authenticated route.
- Added relay and direct target entries for the newly approved fossil ID while retaining the prior node target.
- PR [#88](https://github.com/willjasen/rawrarr/pull/88) merged as `0888445720de5caf348e7b393099b39b0235d5cd`; its required Basic script tests passed.
- PR [#91](https://github.com/willjasen/rawrarr/pull/91) merged as `718383f2ddc6d0a982a2b183deb5b0ac967311ac`; its required Basic script tests passed. Deployment classifier selected coordinator only.
- Deployed that exact published revision to roarlay as image `sha256:b26cf9bb1c5f0f0d405373b7201dfe295a2b7934785161cfe9fad80b646ad606` (`linux/arm64`). The protected PostgreSQL dump at `~/Library/Application Support/Rawrarr/backups/pre-fossil-routes-718383f-20261026/coordinator.dump` passed `pg_restore --list`. Only roarlay was recreated with `--no-deps`; PostgreSQL and Tailscale remained running with zero restarts.
- After deployment, SQL showed the newly approved metadata node had checked in within 90 seconds. From roarlay, the MacBook Tailscale Serve `/health` and `/direct-api/api/direct/health` returned HTTP 200; direct health reported `status: ok`. The coordinator Serve route still maps HTTPS 443 `/` to `http://127.0.0.1:8080`, external HTTPS `/health` responds, and the roarlay deployment lock was removed.
- Verified the signed-in Chrome Devices page after deployment: the MacBook Pro card shows “Online · 5 ms,” the direct route is healthy, and its fossil avatar is visibly rendered. The avatar seed request uses the coordinator relay, whose target now matches the approved ID. Thus the owner browser can use direct access in this internal session, while the configured relay remains available when direct Tailscale access is absent.
- Focused checks passed: 29 Node tests across coordinator workspace and direct route suites; JavaScript syntax check; `git diff --check`.
- The internal coordinator target-map assertion passed in the workspace suite (27/27), `docker compose ... config --quiet` passed with placeholder protected-file paths, and `git diff --check` passed.
- The isolated OrbStack regression runner passed Node 129/129, Playwright 1/1, and Rust 82/82. Python integration tests were skipped because their disposable infrastructure fixtures are not included in that runner.

## Remaining work

The coordinator status and route fixes are deployed, and the node heartbeat, direct endpoint, coordinator relay configuration, and authenticated browser card/avatar are verified. No fossil service, node data, or trust pin was changed.
