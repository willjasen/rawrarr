# Mac mini storage-node deployment plan

Status: planned, revised 2026-10-18. No remote inspection, deployment, drive scan, or network publication has been performed for this task.

## Confirmed scope

This MacBook's existing local deployment is the **coordinator**: web UI/API, sign-in, and PostgreSQL. The Mac mini will run a PostgreSQL-free **storage** worker in OrbStack, read `/Volumes/silo/Media/data`, and join that coordinator's existing instance through a device-code linking flow. Coordinator and storage can coexist for other installations, but are separate here.

## Placement

| Component | Mac mini location | Container boundary |
| --- | --- | --- |
| Media | `/Volumes/silo/Media/data` | `/media`, read-only |
| Authorized shared app data | `~/Library/Application Support/rawrarr/shared-data` | `/shared`; scope provisioned after approval |
| Encrypted scan metadata | Within shared metadata or an approved upload spool | `/metadata` aliases `/shared/metadata` in the current image |
| Device key, identity, scan/queue state | Dedicated OrbStack local volume | `/data`; never synchronized |
| User passkeys, sessions, PostgreSQL | Remain on the MacBook coordinator | Not deployed to storage-only Mini |
| Age private ID | Owner-only local file outside checkout/shared data, entered directly on the Mini | Read-only secret mount; never delivered by linking |
| Tailscale identity | Host or dedicated sidecar state, to be verified | Never shared between nodes or stored in `/shared` |

Use Compose project `rawrarr-local` on the Mini's own OrbStack engine. A matching Compose project name does not create Rawrarr instance membership. The scanner needs the approved public age recipient to encrypt; it does not need the user's private age identity. Files or a lightweight local store may hold worker progress; no PostgreSQL requirement is established.

## Separate local age setup

The owner will enter the existing age private ID directly on the Mini using a planned local command with hidden input. It will save a protected file and mount it read-only into the worker. This is separate from the coordinator URL/code flow: no private identity traverses that flow or the coordinator. Validate the key locally against the approved instance recipient without printing it. See [local age provisioning](coordinator-linking-plan.md#local-age-identity-provisioning-planned) for handling and acceptance requirements. The setup command remains to be implemented.

## Shared-folder setup with Syncthing (required, planned)

Run one dedicated Rawrarr Syncthing container in OrbStack on each Mac. A native Syncthing process could perform the same function, but containers are the selected packaging for this deployment. Inventory existing Syncthing instances first so the new setup does not duplicate a folder or take another service's ports. The current base Compose does not include a Syncthing service; isolated Syncthing tests do not establish a live shared-folder deployment.

On each Mac, bind the same host directory into both its Rawrarr container and its Syncthing container:

| Host | Host folder | Rawrarr mount | Syncthing folder path |
| --- | --- | --- | --- |
| MacBook coordinator | `~/Library/Application Support/rawrarr/shared-data` (verify current mount before reuse) | `/shared` | `/shared` |
| Mac mini storage | `~/Library/Application Support/rawrarr/shared-data` | `/shared` | `/shared` |

These are two local copies synchronized between Macs, not one network-mounted filesystem. Syncthing configuration, index, device certificate/private key, and API key each persist in a separate node-local volume outside `/shared`. Generate distinct Syncthing identities; never clone their config volumes. Do not mount `/media`, the private age-ID file, Rawrarr `/data`, or PostgreSQL into Syncthing. Mount only the dedicated shared subtree, not its parent application-support directory.

### Provisioning order

1. Back up the coordinator's existing shared folder and inspect both targets. Use the coordinator's current instance manifest and public age recipient as authoritative; start the Mini's replica empty. Do not start an independent Rawrarr instance in that empty folder before initial sync. Preserve the existing `instance_id`; map it to one stable Syncthing folder ID used identically on both peers. Syncthing's folder ID is not the Rawrarr instance ID.
2. Start separate Syncthing containers with version-pinned official images, persistent local config, and compatible UID/GID permissions. Verify that Syncthing can read files Rawrarr creates and that Rawrarr can read synced files. Do not recursively loosen unrelated host permissions or mount secrets to fix access.
3. After Rawrarr owner approval, explicitly configure the two verified Syncthing device IDs as peers and share only this instance's folder. Rawrarr linking must securely bind/provision those peer identities or require an explicit verified setup step; a device code alone does not currently configure Syncthing. Never auto-accept arbitrary peers/folders. Keep Syncthing API credentials local to each node rather than transferring an administrator API key through the linking page.
4. Use send-and-receive on both replicas because the coordinator publishes settings and the storage worker publishes metadata. Keep configuration edits coordinator-owned and worker output device-namespaced; Syncthing does not enforce per-file writers or resolve application-level conflicts. This setup is for the owner's trusted full-replica nodes, not selective access for untrusted users. Stop and investigate conflicts rather than force-overriding them.
5. Verify initial sync is complete, the instance manifest/recipient agree, and the folder is error-free before enabling worker scans or local private-ID validation. Provision that private ID separately on the Mini as planned; it must never appear in the replicated tree.

### Network and acceptance

The coordinator HTTPS Serve/Service endpoint carries Rawrarr linking/control; it does not automatically carry Syncthing's sync protocol. Establish direct peer addresses over the verified private Tailscale path, typically TCP 22000, with narrowly scoped access. UDP 22000 is optional for QUIC; LAN discovery on UDP 21027 is unnecessary with explicit peer addresses. Do not assume discovery or macOS host routing works inside OrbStack containers. Verify the actual container-to-container path and avoid public port forwarding. If choosing Tailscale-only transfer, disable global discovery, relays, and NAT traversal so sync does not silently use another route.

Keep Syncthing GUI/API port 8384 private to its app network or a loopback-only admin binding, with authentication enabled. Do not publish it through the Rawrarr coordinator Service or expose it to the tailnet broadly. Inspect existing bindings before selecting admin ports. `/shared` contains age-encrypted configuration/metadata and public manifest/recipient material; existing allowlisted compressed logs are not all individually age-encrypted, so do not describe the entire folder as ciphertext-only. Never send the new human device codes to shared logs.

Acceptance requires verified peer identity, same folder mapping and instance ID, connected/up-to-date status with no errors/conflicts, and real file propagation in both directions. Use an isolated test folder first, then a non-secret disposable probe in a documented ignored-by-Rawrarr location under the actual shared tree; compare hashes and remove it after verification. Verify an encrypted metadata record from the Mini reaches the coordinator, and test permission compatibility, restart persistence, disconnect/reconnect catch-up, and deletion propagation using disposable probes. Confirm excluded secrets, local state, and media are not mounted/shared. File sync success is separate from coordinator catalog ingestion.

Syncthing is synchronization, not backup: deletions propagate. Keep backups outside the synchronized folder; consider receiver-side versioning as extra recovery, not a substitute. If setup fails, pause the new folder/peer relationship without deleting either replica or resetting existing Syncthing instances. On device revocation, remove both Rawrarr authorization and Syncthing peer/folder access; neither action recalls files or age keys already copied.

Implementation can build on `scripts/tests/test_rawrarr_syncthing_orbstack.py`, but must add persistent Compose packaging and test the real two-Mac path. See the [official Docker image guide](https://github.com/syncthing/syncthing/blob/main/README-Docker.md) and [Syncthing firewall documentation](https://docs.syncthing.net/users/firewall.html).

## Current capability and gaps

The full app and base Compose still start web/auth and PostgreSQL. The existing `storage-scan-once` command bypasses those and writes encrypted scan metadata; it is the database-free foundation, not yet a persistent enrolled worker. Do not use the existing base Compose unchanged as the proposed storage-only deployment. Role settings, worker packaging, enrollment routes, authenticated commands, and coordinator catalog consumption remain planned.

The current scanner has one root, skips hidden entries and symlinks, and caps traversal at depth 64, 250,000 entries, and 10,000 media items. Report issues/truncation; do not treat capped output as a full library inventory. Multi-library scanning and cross-node reconciliation remain planned.

## Deployment sequence

1. **Inspect hosts read-only.** Confirm the Mini host, CPU/macOS, OrbStack/Docker context, existing services/volumes, available space, and the silo volume identity, filesystem, mount, and app-user readability. Check existing Tailscale identities, DNS, and Serve configuration on both Macs without altering other services. If publishing a Rawrarr local HTTP endpoint, first check port 7297 ownership and preserve the required `127.0.0.1:7297` binding.
2. **Prepare the coordinator.** Back up its PostgreSQL and consistent shared data before changes. Keep its current local browser access and sessions intact. Implement the [device-code linking contract](coordinator-linking-plan.md), plus a tested WebAuthn strategy for its Tailscale HTTPS origin. Do not assume localhost passkeys work at a new RP hostname.
3. **Establish private connectivity.** Follow the [Tailscale plan](tailscale-integration-plan.md). Prefer a stable coordinator Service name with a single coordinator backend. Verify reachability from inside the Mini's OrbStack worker container, not only from the macOS host. No media traffic, PostgreSQL exposure, or public Funnel.
4. **Package a storage-only worker.** Separate it from the base web/auth Compose stack. Persist a unique device key and local scan state. Supply the trusted coordinator URL as non-secret configuration. Implement bounded retry/polling, graceful stop, health/readiness, and pending/offline states. Do not copy coordinator `/data`, sessions, or database files onto the Mini.
5. **Link the node.** Worker logs the coordinator `/devices/link` URL and a short-lived code. The owner signs in there, enters the code, reviews the device, and approves. Verify key-bound enrollment and distinct device IDs with the same authorized instance. Provision the shared folder using the required Syncthing setup above, retaining the existing manifest/recipient. Never create a new instance then merge it into the old one.
6. **Validate the external mount.** Set the exact media source to `/Volumes/silo/Media/data`, `/media` read-only, and `bind.create_host_path: false` in the future worker Compose. The existing media overlay lacks that missing-path guard. Check the actual volume identity and a known existing entry, not just directory existence. Verify mount metadata and app-user reads; do not test write blocking by writing to real media.
7. **Verify the real result.** Start with a bounded fixture/sample, then scan the approved root. Confirm counts, categories, no unexpected permission errors, encrypted outputs, and coordinator catalog ingestion attributed to the Mini. Confirm no PostgreSQL or user-passkey storage exists on the worker. Catalog ingestion is a required implementation milestone, not proved by successful sync alone.
8. **Exercise failure and recovery.** Use fixtures first for missing-drive startup/remount, code expiry, revocation, coordinator sleep, retries, worker restart, and persistent identity. Missing media must be unavailable rather than reconciled as mass deletion; implement this before unattended use. Coordinate any real silo disconnect/reboot with its other consumers. Verify OrbStack restart and host login/reboot behavior before calling the node unattended-ready.

## Rollback and remaining work

Stop only the new worker and pause its new sync relationship if validation fails; retain volumes and evidence. Revoke its enrollment on the coordinator. Do not run `down -v`, delete media, or reset unrelated Serve configuration. Roll back only task-owned proxy and app changes to recorded settings; restore backups only with a reviewed scope and synchronization paused.

Remaining: host preflight, storage packaging/worker lifecycle, device-code enrollment, HTTPS/WebAuthn compatibility, persistent Syncthing containers and verified shared-folder replication, scoped transport, missing-drive handling, and catalog consumption. The plan does not authorize claiming any of these as deployed.

References: [node types](node-types.md), [data domains](data-domains.md), [OrbStack bind mounts](https://docs.orbstack.dev/docker/file-sharing), and [Compose volumes](https://docs.docker.com/reference/compose-file/services/#volumes).
