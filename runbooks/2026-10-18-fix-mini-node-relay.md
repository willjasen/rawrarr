# Restore the Mac mini node relay route

## Objective and initial state

Make the approved Mac mini storage page available from the MacBook coordinator through its registered-node relay. The coordinator listed `mac-mini-storage` as online but without a route. Read-only inspection found the Mini storage container restarting after enrollment HTTP 502s, a preserved Tailscale state directory but no `rawrarr-mini-tailscale` container, and an obsolete storage URL pointing to that absent sidecar's local coordinator bridge. The Mini's separate Caddy coordinator proxy still existed, and its Compose file already used the Tailscale SOCKS5 endpoint. The MacBook coordinator had an empty relay map and no relay SOCKS client setting.

## Actions

- Added persistent SOCKS5 listener configuration to both Mac Tailscale sidecars.
- Added a trusted Mini storage HTTPS target to local coordinator Compose and an explicit SOCKS5 proxy setting to the coordinator relay client. The Docker `metadata` target bypasses SOCKS so the existing same-network metadata route remains direct.
- Added a Mini storage Caddy bridge on the Mini Tailscale network namespace and set the Mini Tailscale Serve endpoint to reverse-proxy that bridge to `rawrarr-mini-storage:8080`.
- Updated storage's coordinator client to use its configured SOCKS5 proxy directly for HTTPS enrollment/heartbeats; the previously present `RAWARR_COORDINATOR_SOCKS5_URL` had not been applied by the Rust client. Restored the Mini Tailscale container from its existing protected state directory, preserving its established tailnet identity. Retained the Mini Rawrarr data volume, shared folder, Syncthing state, media, PostgreSQL volume, and all Tailscale state.
- Updated the coordinator image and recreated only the coordinator service in the existing `rawrarr-local` project, using the protected PostgreSQL password-file path and preserving the database volume.

## Verification

- Recreated the Mini Tailscale and Rawrarr storage containers; Mini storage is healthy with zero restart count. The Mini's local `/health` endpoint returned HTTP 200.
- Confirmed the coordinator can reach `https://rawrarr-mini.risk-mermaid.ts.net/health` through its own Tailscale SOCKS5 listener; response was HTTP 200.
- Confirmed the public coordinator Serve endpoint `https://rawrarr.risk-mermaid.ts.net/health` returns HTTP 200.
- The signed-in coordinator workspace listed `mac-mini-storage` as online without the prior “route not configured” state.
- Focused coordinator workspace tests passed (5/5), including configured tailnet SOCKS routing and accessible online/offline indicators. Compose configuration validation and `git diff --check` passed.
- Mini storage was rebuilt natively for `linux/arm64`; coordinator image was rebuilt locally and only its container was recreated. Existing state and data volumes were retained.

## Recovery and remaining work

- If Mini Serve must be disabled, remove only its `/` HTTPS handler with the matching `tailscale serve --https=443 off` command; do not reset all Serve configuration.
- Restore prior Compose files from protected backups if required. Do not remove Docker volumes or alter the read-only media mount.
- Protected pre-change Compose copies are in `~/Library/Application Support/rawrarr/deploy/backups/2026-10-18-fix-mini-node-relay/`. Restore only the specific manifest needed; keep the existing state directories and volumes.
- The route now works end to end. No deployment step remains for this fix.
