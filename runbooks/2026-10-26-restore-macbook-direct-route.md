# Restore the MacBook direct route

## Objective and tracking

Restore the MacBook Pro `fossil` node's browser direct route after the coordinator switched its direct health probe to the shortened endpoint. Tracked in [issue #141](https://github.com/willjasen/rawrarr/issues/141).

## Initial state

- The signed-in Devices card shows the MacBook `fossil` as **Online · Relay**. The user reports it was Direct before adding the Mac mini node.
- GitHub `main` was `d7e3df88816d66c44679adb326e2daa53b554019`. It probes `/direct-api/health` and opens `/direct-api/session`.
- From this workstation, the MacBook Tailscale Serve route returns HTTP 404 for `/direct-api/health` and HTTP 200 for the old `/direct-api/api/direct/health` path. The Mac mini responds HTTP 200 on the new path.
- The MacBook node was not updated when the Mac mini direct endpoint path was shortened, leaving its handler out of step with the coordinator.

## Actions and verification

- Built the final MacBook image from published merge commit `38b4d482f01e4578aebe10c66ff65fa681bdda48` (PR [#137](https://github.com/willjasen/rawrarr/pull/137)), which includes the shortened direct endpoints and the Syncthing peer-completion fix from PR #138. This commit is an ancestor of current `main`; used it to avoid introducing the newer group-age runtime rollout, whose runbook says live multi-node deployment remains unverified.
- An initial build from commit `9068edb4bc3cb391bc9b2ddf7f54081f6c398242` was replaced before task completion after confirming it predated PR #138. It is retained as `rawrarr:rollback-fossil-direct-route-9068` (`sha256:3852569825daa584a0aa355cfc583a10a9b64d6d7c8e7377c886f51a9efeac48`). The pre-task image is also retained as `rawrarr:rollback-fossil-direct-route-20261026` (`sha256:2d5b033cf2d4e0071cd6ab75c48a0ee6ea422d9d3eb35e8421afe84c608695cc`).
- Atomically claimed the MacBook fossil deployment lock and built the final `rawrarr:local` image from the published merge commit; image ID `sha256:358feaefb3698723c5c991e41bb01eed13b796f3565413cf12963134079fc6e1`, labeled with its source revision.
- Recreated only `fossil` in Compose project `rawrarr-node-internal` with `--no-deps`. MacBook Valkey and Tailscale remained running; the Tailscale Serve handlers still map `/` to `http://127.0.0.1:8080` and `/direct-api` to `http://127.0.0.1:8082`.
- The MacBook HTTPS `/health` and `/direct-api/health` endpoints both returned HTTP 200; direct health returned `status: ok`. An unauthenticated POST to `/direct-api/session` returned HTTP 401, and the superseded `/direct-api/api/direct/health` path returned HTTP 404. The `fossil` container is healthy with zero restarts and runs the final image.
- Read the already-open signed-in Devices page after the final deployment. It reports MacBook Pro **Direct (11 ms)** and Mac mini **Direct (16 ms)**. The coordinator and Mac mini were not changed. The fossil deployment lock was removed after verification.
- The route-code diff from its parent commit classifies as coordinator and node runtime code. The live Devices page already exercised the new probe successfully once the MacBook endpoint was updated, so the stale MacBook node alone needed recreation.
- `git diff --check` passed. No source code changed in this task; the direct-path code came from the already published commit above.

## Final state and remaining work

The MacBook card is Direct again and the endpoint mismatch is resolved. Syncthing health remains unavailable on both cards even with the peer-completion fix deployed; that needs separate diagnosis.
