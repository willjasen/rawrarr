# Repair stale coordinator routes for internal devices

Issue: [#204](https://github.com/willjasen/rawrarr/issues/204)

## Finding

Both enrolled metadata nodes are checking in, but the Mac mini coordinator's relay and direct target maps reference an obsolete node ID. Current registered nodes are the MacBook Pro and Mac mini. Their direct API health endpoints both respond over their tailnet HTTPS names, and the MacBook endpoint permits the coordinator origin through CORS.

## Change

The active roarlay environment maps were stale: both referenced an obsolete MacBook node ID. Updated only `RAWARR_NODE_RELAY_TARGETS` and `RAWARR_NODE_DIRECT_TARGETS` in the Mac mini's local `docker-compose.internal-mini.yml` to map the current MacBook Pro and Mac mini registrations to their corresponding tailnet hostnames. The source file remains host-local; no runtime configuration or secrets were added to the repository.

Atomically claimed `.container-rebuild-roarlay.lock`, validated the existing Compose file set with its protected secret-file paths, and recreated only `roarlay` with `--no-deps --pull never`. PostgreSQL, the Tailscale sidecar, subscriber services, and all node-local and synchronized data were preserved.

## Verification

- The running roarlay container reports `healthy`, zero restarts, and the two corrected direct and relay mappings.
- Both node `/direct-api/health` endpoints returned JSON status `ok` and `Access-Control-Allow-Origin: https://rawrarr.risk-mermaid.ts.net`.
- Coordinator HTTPS `/health` returned HTTP 200 and identified Roarlay. Tailscale Serve JSON still maps HTTPS 443 `/` to `http://127.0.0.1:8080`.
- Released the roarlay deployment lock after verification.
- The authenticated Devices page itself was not available in the current browser session, so visual card labels were not directly observed. Given its route probe logic, both cards should now report **Direct** when refreshed.
