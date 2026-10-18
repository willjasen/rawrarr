# Reconnect the MacBook metadata namespace

## Objective and initial state

Restore the MacBook metadata node's direct Tailscale route after Devices showed it as relay-only. The metadata container was healthy in isolation but used a removed Tailscale container ID as its network namespace; the current `rawrarr-macbook-tailscale` namespace could not reach ports 8081 or 8082, and the public direct health probe returned HTTP 502. The active Compose project was `rawrarr-local`; port 7297 and coordinator health were already working.

Related history: [Implement the Tailscale direct node route](2026-10-18-implement-tailscale-direct-node-route.md), [Fix the encrypted library relay 502](2026-10-18-fix-library-relay-502.md), and [Share the device route status checker](2026-10-18-share-device-route-status-checker.md).

## Actions

- Confirmed the metadata network mode referenced a nonexistent namespace ID, while the running Tailscale container had a different ID.
- Claimed the metadata deployment marker under `~/Library/Application Support/Rawrarr/` and recreated only the `metadata` service in `rawrarr-local` using the coordinator and MacBook Tailscale Compose overlays. Passed the existing protected PostgreSQL password-file path without reading its contents.
- Confirmed Docker attached metadata to the current Tailscale container namespace; did not rebuild or restart roarlay or Tailscale.

## Verification

- Metadata container reported healthy and its network mode matched the current Tailscale container ID.
- From roarlay's shared namespace, metadata `/health` returned HTTP 200 and `/api/direct/health` returned HTTP 204.
- Public `https://rawrarr.risk-mermaid.ts.net/direct-api/api/direct/health` returned HTTP 204.
- Local port 7297 health and public coordinator `/health` both returned HTTP 200.
- Tailscale Serve readback preserved `/` -> `http://127.0.0.1:8080` and `/direct-api` -> `http://127.0.0.1:8082`.
- Removed the task-owned metadata deployment marker after health and route verification.

## Recovery and remaining work

If metadata is recreated against an obsolete namespace again, reattach it using all active Compose overlays. Do not reset Tailscale Serve or delete volumes. The Mac mini storage node remains relay-only by design; this repair restores the MacBook metadata direct route.
