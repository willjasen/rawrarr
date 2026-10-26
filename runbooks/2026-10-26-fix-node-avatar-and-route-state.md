# Keep node avatars visible and route health truthful

## Objective and tracking

Fix recurring newly enrolled node cards that show a broken profile image and a green “Online · Route unavailable” indicator. Tracked in [issue #181](https://github.com/willjasen/rawrarr/issues/181).

## Cause

- The device avatar seed is derived by fetching the node's public age recipient through the authenticated coordinator relay, but the fossil artwork itself is rendered by the coordinator.
- The UI returned before requesting that recipient whenever the node had no relay route. A missing route map therefore also hid an otherwise renderable avatar.
- The state pill used heartbeat freshness for its green class even when the live browser route probe failed. The text was technically separate, but the green treatment looked healthy.
- Internal Compose maps are keyed by node IDs, which can change on re-enrollment; a route-map omission recurs unless deployment verifies the current IDs.

## Changes

- Always attempt to load the node recipient/avatar regardless of heartbeat or relay state. The authorized recipient route can fail safely; when it succeeds, the coordinator serves the deterministic image locally.
- Keep heartbeat text (`Online`) and route text (`Direct`, `Relay`, or `Route unavailable`) separate. Style the online plus unavailable-route combination amber, while a stale heartbeat remains red.
- The internal Compose file still requires the current approved IDs to be kept in its relay/direct maps. Before each internal coordinator deployment, compare those maps with the live approved node records and probe each mapped route; fail verification on missing or stale IDs. Never infer tailnet endpoints from node-supplied values.

## Verification

- `node --test scripts/tests/test_rawrarr_node_avatar.mjs scripts/tests/test_rawrarr_coordinator_workspace.mjs scripts/tests/test_rawrarr_direct_route.mjs` — passed, 34/34.
- `git diff --check` — passed.
- Internal live node/map comparison, container deployment, route health, and authenticated browser verification are pending because this host's Tailscale CLI cannot load preferences and browser access was denied by the origin safety review.

## Deployment follow-up

After the change is published to GitHub `main`, follow [the internal coordinator deployment runbook](2026-10-22-deploy-internal-mini-coordinator.md): claim the roarlay deployment lock, build the exact published source for `linux/arm64`, preserve PostgreSQL/Tailscale state, recreate only roarlay, verify health and the 443 Serve route, then check every currently enrolled node's relay/direct targets and image responses. Keep the fossil service unchanged for this coordinator-only UI fix.
