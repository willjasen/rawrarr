# Storage media replication

## Objective and initial state

Add an explicit Syncthing setup path for sharing selected media folders from the Mac mini apex to a second MacBook apex node. The existing `rawrarr-shared` folder carries application data; storage nodes mount `/media` read-only in Rawrarr, and Syncthing did not initially mount media. The user selected only top-level music folders beginning with A or a, with the Mini as send-only source and the MacBook as receive-only destination. The Mini had two matching folders totaling about 3.3 GiB; the MacBook had about 139 GiB free.

Related context: [Syncthing peer sync](2026-10-17-rawrarr-syncthing-peer-sync.md), [Mac mini storage deployment](2026-10-18-mac-mini-storage-node-deployment.md).

## Work and verification

- Added `docker-compose.syncthing-media.yml` as a bind-mount overlay; the Mini Syncthing container mounts its existing media tree read-only and the MacBook Syncthing container mounts a separate replica directory writable. Neither Rawrarr worker nor roarlay gained a writable media mount. Preserved Syncthing state volumes, existing `rawrarr-shared` folders, Docker project `rawrarr-local`, and all other containers.
- Added `scripts/share_storage_media.py`: read-only status and dry-run by default, explicit local/peer device IDs, safe `/media` paths, existing-folder overlap checks, no auto-accept, and `--apply` for send-only source or receive-only destination. Added a read-only tree hash script and focused tests. Configured two distinct media folder IDs on the already-paired Syncthing devices. The destination was empty before pairing. Both reached idle with zero needed files/bytes and errors.
- Path-and-content SHA-256 comparison matched for both folders: 48 files / 1,339,677,275 bytes and 69 files / 2,214,265,670 bytes, respectively. The matching tree digests were `13ba9cf2847ab96c7b121108be3b4172287adad434b8e4bca3eac9b75b179b74` and `5373034bd29080b840d5c1cdcff4299c8eaed2dcbd157c295caae8d8ac6937a7`. No media files were added to Git.
- After that transfer, added Tailscale Serve TCP 22000 forwarding to each host's Syncthing container while preserving both HTTPS 443 handlers. Added `docker-compose.syncthing-tailnet.yml` with each Syncthing process using its own userspace Tailscale SOCKS5 proxy and `ALL_PROXY_NO_FALLBACK=1`. Added `scripts/lock_syncthing_to_tailscale.py`, pinned each peer to the other's Tailscale IPv4 TCP address, and disabled global/local discovery, relays, and NAT traversal. Both peers read back TCP rather than relay connections, with the MacBook client connected to the Mini tailnet IP and no restart required. A temporary source probe file arrived with the same SHA-256 and was removed at both ends after the source deletion. The `/shared` folder and both media folders remained idle with zero needed files and errors.
- The proxy initially made the image's localhost `curl` health check use SOCKS5. Added `NO_PROXY` and `no_proxy` for localhost in the tailnet Compose overlay, then recreated only the two Syncthing containers. Final health is recorded below.
- Added `docker-compose.macbook-storage.yml` for a distinct MacBook storage worker, with its own `/data` volume, synchronized local media mounted read-only, and no host port. It submitted the `macbook-storage` request to roarlay and is waiting for explicit owner approval. The roarlay relay map includes its future loopback listener. Roarlay was recreated with the existing Compose project and original file set; its container is healthy, local 7297 responds, and Tailscale Serve HTTPS 443 `/` still targets `http://127.0.0.1:8080` with a successful `/health` response identifying roarlay.

### Final verification

Final transport and health read-back: both Syncthing containers healthy after the localhost proxy exemption; `rawrarr-shared` idle on both peers with 4,486 local/global files, zero needed bytes, and zero errors; both music folders idle with 48 and 69 matching files. Each peer reports `tcp-client`/`tcp-server`, a static 100.64/10 peer address, disabled discovery/relays/NAT, and `restartRequired=false`. Both Tailscale Serve JSON documents retain HTTPS 443 and TCP 22000 forwards to the intended local Syncthing containers. The roarlay container and tailnet `/health` route are healthy. Owner approval of `macbook-storage` remains outstanding.

## Recovery and remaining work

Keep source and destination media and both Syncthing state volumes. To pause replication, pause the two media folders in Syncthing on both peers; do not delete folders or volumes. To undo tailnet-only transport, first inspect current Serve handlers, then deliberately restore the prior Syncthing proxy/config values from protected node-local backups or the recorded pre-change values; do not reset Serve because the HTTPS routes are independent. If the tailnet route fails, `ALL_PROXY_NO_FALLBACK=1` intentionally stops sync rather than falling back to a public relay.

The MacBook apex remains pending until the owner approves it in roarlay Devices. Verify its healthy listener, media scan, and catalog appearance after approval. Media synchronization alone does not activate that node.
