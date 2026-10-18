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

## Current capability and gaps

The full app and base Compose still start web/auth and PostgreSQL. The existing `storage-scan-once` command bypasses those and writes encrypted scan metadata; it is the database-free foundation, not yet a persistent enrolled worker. Do not use the existing base Compose unchanged as the proposed storage-only deployment. Role settings, worker packaging, enrollment routes, authenticated commands, and coordinator catalog consumption remain planned.

The current scanner has one root, skips hidden entries and symlinks, and caps traversal at depth 64, 250,000 entries, and 10,000 media items. Report issues/truncation; do not treat capped output as a full library inventory. Multi-library scanning and cross-node reconciliation remain planned.

## Deployment sequence

1. **Inspect hosts read-only.** Confirm the Mini host, CPU/macOS, OrbStack/Docker context, existing services/volumes, available space, and the silo volume identity, filesystem, mount, and app-user readability. Check existing Tailscale identities, DNS, and Serve configuration on both Macs without altering other services. If publishing a Rawrarr local HTTP endpoint, first check port 7297 ownership and preserve the required `127.0.0.1:7297` binding.
2. **Prepare the coordinator.** Back up its PostgreSQL and consistent shared data before changes. Keep its current local browser access and sessions intact. Implement the [device-code linking contract](coordinator-linking-plan.md), plus a tested WebAuthn strategy for its Tailscale HTTPS origin. Do not assume localhost passkeys work at a new RP hostname.
3. **Establish private connectivity.** Follow the [Tailscale plan](tailscale-integration-plan.md). Prefer a stable coordinator Service name with a single coordinator backend. Verify reachability from inside the Mini's OrbStack worker container, not only from the macOS host. No media traffic, PostgreSQL exposure, or public Funnel.
4. **Package a storage-only worker.** Separate it from the base web/auth Compose stack. Persist a unique device key and local scan state. Supply the trusted coordinator URL as non-secret configuration. Implement bounded retry/polling, graceful stop, health/readiness, and pending/offline states. Do not copy coordinator `/data`, sessions, or database files onto the Mini.
5. **Link the node.** Worker logs the coordinator `/devices/link` URL and a short-lived code. The owner signs in there, enters the code, reviews the device, and approves. Verify key-bound enrollment and distinct device IDs with the same authorized instance. Provision only the needed app-data access, retaining the existing manifest/recipient. Never create a new instance then merge it into the old one.
6. **Validate the external mount.** Set the exact media source to `/Volumes/silo/Media/data`, `/media` read-only, and `bind.create_host_path: false` in the future worker Compose. The existing media overlay lacks that missing-path guard. Check the actual volume identity and a known existing entry, not just directory existence. Verify mount metadata and app-user reads; do not test write blocking by writing to real media.
7. **Verify the real result.** Start with a bounded fixture/sample, then scan the approved root. Confirm counts, categories, no unexpected permission errors, encrypted outputs, and coordinator catalog ingestion attributed to the Mini. Confirm no PostgreSQL or user-passkey storage exists on the worker. Catalog ingestion is a required implementation milestone, not proved by successful sync alone.
8. **Exercise failure and recovery.** Use fixtures first for missing-drive startup/remount, code expiry, revocation, coordinator sleep, retries, worker restart, and persistent identity. Missing media must be unavailable rather than reconciled as mass deletion; implement this before unattended use. Coordinate any real silo disconnect/reboot with its other consumers. Verify OrbStack restart and host login/reboot behavior before calling the node unattended-ready.

## Rollback and remaining work

Stop only the new worker and pause its new sync relationship if validation fails; retain volumes and evidence. Revoke its enrollment on the coordinator. Do not run `down -v`, delete media, or reset unrelated Serve configuration. Roll back only task-owned proxy and app changes to recorded settings; restore backups only with a reviewed scope and synchronization paused.

Remaining: host preflight, storage packaging/worker lifecycle, device-code enrollment, HTTPS/WebAuthn compatibility, scoped transport, missing-drive handling, and catalog consumption. The plan does not authorize claiming any of these as deployed.

References: [node types](node-types.md), [data domains](data-domains.md), [OrbStack bind mounts](https://docs.orbstack.dev/docker/file-sharing), and [Compose volumes](https://docs.docker.com/reference/compose-file/services/#volumes).
