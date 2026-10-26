# Restore the current MacBook fossil route

## Objective and tracking

Restore direct and coordinator-relay routing for the MacBook Pro fossil shown as “Online · Route unavailable”. Tracked in [issue #181](https://github.com/willjasen/rawrarr/issues/181).

## Cause

- The MacBook fossil container is healthy and currently reports node ID `device-398f0ce833b6d7e2b9e8f760`.
- The coordinator's published `RAWARR_NODE_RELAY_TARGETS` and `RAWARR_NODE_DIRECT_TARGETS` maps do not contain that current ID, although they retain entries for older MacBook enrollments.
- The verified MacBook tailnet endpoint `https://rawrarr-macbook-pro.risk-mermaid.ts.net/direct-api/health` returns HTTP 200 with `status: ok`. Its response and OPTIONS preflight include the coordinator origin `https://rawrarr.risk-mermaid.ts.net` in the CORS headers.

## Changes and verification

- Added the current MacBook ID to both coordinator route maps, preserving existing entries for older enrollments and the Mac mini.
- Added a focused regression assertion that requires the current ID to appear in both maps with the verified MacBook endpoints.
- `node --test scripts/tests/test_rawrarr_coordinator_workspace.mjs` passed 28/28.
- `docker compose -f docker-compose.internal-mini.yml config -q` passed with placeholder values for protected-file paths; no protected files were read or changed.
- `git diff --check` passed.

## Deployment and live verification

- PR [#187](https://github.com/willjasen/rawrarr/pull/187) merged as published `main` commit `233bab0e4f207bb4848467931c273a43c193cd09`. The Mac mini's stored GitHub CLI credential was invalid, so the exact published GitHub tarball was downloaded with authenticated `gh api` on the workstation and transferred over the verified SSH alias into a new, previously unused build directory. Its SHA-256 was `21ab16c32fc080be42d4a7472bbd86cd49883e2513e94bfa08cec4cb5b61a053`.
- The old roarlay image was retained as `rawrarr:rollback-roarlay-pre-233bab0`. Built the new image on the Mac mini from that published archive with `DOCKER_DEFAULT_PLATFORM=linux/amd64`; image ID `sha256:07b6a4c143a76dc55ea186bae0ec72e868333eea90ab9bc9d26b9875ce09deeb` reports `linux/amd64`.
- Recreated only `roarlay` in Compose project `rawrarr-internal-mini` using the published `233bab0` Compose file and existing protected configuration paths. Roarlay is healthy with zero restarts. PostgreSQL remains healthy and the Tailscale sidecar remains running; neither was recreated. The task-owned `.container-rebuild-roarlay.lock` was removed after verification.
- Both live route maps now include the MacBook's current ID `device-398f0ce833b6d7e2b9e8f760`. From the Mac mini, `https://rawrarr-macbook-pro.risk-mermaid.ts.net/direct-api/health` returned HTTP 200 with `status: ok` and `Access-Control-Allow-Origin: https://rawrarr.risk-mermaid.ts.net`.
- `docker exec rawrarr-internal-mini-tailscale-1 tailscale serve status --json` confirms `rawrarr.risk-mermaid.ts.net:443` `/` still proxies to `http://127.0.0.1:8080`; external HTTPS `/health` serves the Roarlay health page.
- Opened the MacBook's direct health URL in the in-app browser after deployment; it returned `{"artworkCacheConfigured":false,"artworkCacheReachable":false,"status":"ok"}`. The signed-in Devices card itself was not rechecked because this browser session is signed out. The browser endpoint, CORS, both live maps, and the coordinator Serve route are verified.
