# Update Mac mini Rawrarr containers

## Objective

Bring the Mac mini Rawrarr storage worker and locally built relay proxies up to the current project source and refresh upstream images where a newer stable image is available. Preserve `/data`, `/shared`, `/media`, Syncthing state, and Tailscale state.

## Initial state

- Mini services are `rawrarr-mini-storage`, `rawrarr-mini-storage-proxy`, `rawrarr-mini-tailscale`, `rawrarr-mini-coordinator-proxy`, and `rawrarr-mini-syncthing`; storage and Syncthing report healthy.
- Storage image was built at 14:55 EDT, while the current MacBook image was rebuilt from newer source at 15:13 EDT.
- Tailscale uses the `stable` tag and its current digest matches the upstream tag. Syncthing is pinned to 2.1.5, the latest stable release; 2.1.6 is only a release candidate.
- The Mini storage deployment marker is present. Other Rawrarr tasks are active, so do not alter the marker or restart its network dependencies until it clears.
- Existing deployment sources live under `~/Library/Application Support/rawrarr/deploy`; previous procedures use `/usr/local/bin/docker` and add `/usr/local/bin` to SSH `PATH` for the macOS credential helper.

## Actions

- Confirmed the existing storage marker had no process holding it and matched the timestamp of a prior interrupted deployment attempt. The user directed me to continue; replaced it atomically with this task's marker.
- Backed up the Mini deployment source at `~/Library/Application Support/rawrarr/deploy/backups/rust-arr-before-container-update-20261018-1526.tar.gz`.
- Copied the current project `Cargo.toml`, `Cargo.lock`, `Dockerfile`, `migrations/`, and `src/` into the Mini deployment source. Verified the deployed `coordinator-workspace.mjs` SHA-256 matches the project copy.
- Rebuilt the storage image with `--pull --no-cache` to refresh its Debian packages, then recreated only the storage service. Preserved its named `/data` volume, `/shared` bind, and read-only `/media` bind.
- Rebuilt the shared Caddy proxy image with `--pull` and recreated only `coordinator-proxy` and `storage-proxy`. Released their markers after verification.
- Checked upstream releases: Tailscale stable is v1.104.1 and the running Mini image reports v1.104.1. Syncthing v2.1.5 remains the latest stable release; v2.1.6 was only a release candidate, so its pinned image was left unchanged.

## Verification

- Storage container is `running/healthy`; storage `/health` returned HTTP 200 and direct API health returned HTTP 204.
- Both Caddy proxies are running on the rebuilt `rawrarr-coordinator-proxy:local` image; `caddy version` reports v2.10.0.
- Syncthing remains `running/healthy`, and its local health endpoint returned `OK`.
- Tailscale remains running as v1.104.1. Serve readback preserves root `/` -> `http://127.0.0.1:8082` and `/direct-api` -> `http://127.0.0.1:8083`.
- Public Mini health returned HTTP 200; public direct health returned HTTP 204.
- No active scan log entries were present before the storage recreation.

## Recovery and final state

Restore the saved source archive and rebuild the storage image if rollback is needed. Preserve the three storage mounts, Syncthing state volume, and Tailscale state. All five Mini containers are running; storage and Syncthing report healthy. Storage and proxy deployment markers were removed after checks.
