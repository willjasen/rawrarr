# Restore the MacBook direct route

## Objective and tracking

Restore the MacBook Pro `fossil` node's browser direct route after the coordinator switched its direct health probe to the shortened endpoint. Tracked in [issue #141](https://github.com/willjasen/rawrarr/issues/141).

## Initial state

- The signed-in Devices card shows the MacBook `fossil` as **Online · Relay**. The user reports it was Direct before adding the Mac mini node.
- GitHub `main` was `d7e3df88816d66c44679adb326e2daa53b554019`. It probes `/direct-api/health` and opens `/direct-api/session`.
- From this workstation, the MacBook Tailscale Serve route returns HTTP 404 for `/direct-api/health` and HTTP 200 for the old `/direct-api/api/direct/health` path. The Mac mini responds HTTP 200 on the new path.
- The MacBook node was not updated when the Mac mini direct endpoint path was shortened, leaving its handler out of step with the coordinator.

## Actions and verification

- Built the MacBook image from published commit `9068edb4bc3cb391bc9b2ddf7f54081f6c398242` (`Shorten direct API bootstrap routes`), which is an ancestor of the fetched and GitHub-reported `main`. Used this focused published revision because the current `main` tip also contains the separate group-age runtime rollout, whose runbook says live multi-node deployment remains unverified.
- Atomically claimed the MacBook fossil deployment lock and tagged its prior image `sha256:2d5b033cf2d4e0071cd6ab75c48a0ee6ea422d9d3eb35e8421afe84c608695cc` as a rollback image. Built `rawrarr:local` from the published commit; resulting image is `sha256:3852569825daa584a0aa355cfc583a10a9b64d6d7c8e7377c886f51a9efeac48`, labeled with that source revision.
- Recreated only `fossil` in Compose project `rawrarr-node-internal` with `--no-deps`. MacBook Valkey and Tailscale remained running; the Tailscale Serve handlers still map `/` to `http://127.0.0.1:8080` and `/direct-api` to `http://127.0.0.1:8082`.
- The new `https://rawrarr-macbook-pro.risk-mermaid.ts.net/direct-api/health` endpoint returned HTTP 200 with `status: ok`. The `fossil` container is healthy with zero restarts and runs the image built from the recorded commit.
- An unauthenticated POST to `/direct-api/session` returned HTTP 401, confirming the new session handler is present and still requires authorization; the superseded `/direct-api/api/direct/health` now returns 404.
- Read the already-open signed-in Devices page after deployment. It reports MacBook Pro **Direct (8 ms)** and Mac mini **Direct (13 ms)**. The coordinator and Mac mini were not changed. The fossil deployment lock was removed after verification.
- The route-code diff from its parent commit classifies as coordinator and node runtime code. The live Devices page already exercised the new probe successfully once the MacBook endpoint was updated, so the stale MacBook node alone needed recreation.
- `git diff --check` passed. No source code changed in this task; the direct-path code came from the already published commit above.

## Final state and remaining work

The MacBook card is Direct again and the endpoint mismatch is resolved. The Devices cards still show Syncthing health unavailable, which is separate from this direct-route repair.
