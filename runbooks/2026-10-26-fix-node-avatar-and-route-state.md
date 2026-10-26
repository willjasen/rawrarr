# Keep node avatars visible and route health truthful

## Objective and tracking

Fix recurring newly enrolled node cards that show a broken profile image and a green “Online · Route unavailable” indicator. Tracked in [issue #181](https://github.com/willjasen/rawrarr/issues/181).

## Cause

- The preferred device avatar seed is derived by fetching the node's public age recipient through the authenticated coordinator relay, but the fossil artwork itself is rendered by the coordinator.
- When a newly enrolled ID had no route map, the recipient fetch failed and the card image stayed empty. There was no node-ID-based fallback.
- The state pill used heartbeat freshness for its green class even when the live browser route probe failed. The text was technically separate, but the green treatment looked healthy.
- Internal Compose maps are keyed by node IDs, which can change on re-enrollment; a route-map omission recurs unless deployment verifies the current IDs.

## Changes

- Always attempt to load the node recipient regardless of heartbeat or relay state. If the relay fetch fails, hash the stable node ID as a deterministic fallback seed so the coordinator can still render a fossil picture.
- Keep heartbeat text (`Online`) and route text (`Direct`, `Relay`, or `Route unavailable`) separate. Style the online plus unavailable-route combination amber, while a stale heartbeat remains red.
- The internal Compose file still requires the current approved IDs to be kept in its relay/direct maps. Before each internal coordinator deployment, compare those maps with the live approved node records and probe each mapped route; fail verification on missing or stale IDs. Never infer tailnet endpoints from node-supplied values.

## Verification

- `node --test scripts/tests/test_rawrarr_node_avatar.mjs scripts/tests/test_rawrarr_coordinator_workspace.mjs scripts/tests/test_rawrarr_direct_route.mjs` — passed, 34/34.
- `git diff --check` — passed.
- `python3 scripts/tests/test_internal_deploy_targets.py` — passed, 6/6.
- On the Mac mini, the live coordinator had node IDs `device-398f0ce833b6d7e2b9e8f760` and `device-8de78fadda6400a45ca1a30b`; the published internal Compose maps include the latter but not the former. No endpoint was guessed for the unmapped node. The UI now keeps its avatar visible and colors its fresh-heartbeat/unavailable-route state amber.
- Deployed published `main` commit `7a1722e9eeec4e397244df92541b69158f7071de` to roarlay only. The image is `sha256:63890e2e436e5ae6b116c44496e097e4f94e2190f55027e60f094e3e0671d742` (`linux/arm64`). The prior image remains tagged `rawrarr:rollback-roarlay-pre-7a1722e`.
- PostgreSQL backup `/Users/willjasen/Library/Application Support/Rawrarr/backups/pre-node-avatar-7a1722e-20261026/coordinator.dump` is mode 600 and passed `pg_restore --list` inside the PostgreSQL container.
- The installed UI bundle version is `7a1722e9eeec4e397244df92541b69158f7071de`, SHA-256 `4a2553d61822861e5de0b42274e39c86ee3ba39ffc79dd07e1518fb7400399fd`. The mounted workspace HTML contains the amber style and its JavaScript contains the node-ID fallback.
- Roarlay is healthy; PostgreSQL remains healthy and the Tailscale sidecar remains running. Tailscale Serve reads back HTTPS 443 `/` -> `http://127.0.0.1:8080`, and the external HTTPS `/health` request succeeds. The deployment lock was removed after verification.
- Authenticated visual browser verification was not performed because the browser origin safety review denied access from an unrelated local backup-service tab. The live HTML/JS bundle and route were verified directly.

## Deployment follow-up

For a later route repair, identify the node's actual host and verified tailnet endpoint before updating the relay/direct maps. Keep the fossil service unchanged for this coordinator-only UI fix.
