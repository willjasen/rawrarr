# Back up the internal coordinator on do3

## Objective and issue

Create and verify a protected, recoverable disk backup of the internal coordinator and its private analytics services. Tracked by [issue #299](https://github.com/willjasen/rawrarr/issues/299). The public `rawrarr-roarlay-prod` project is deliberately outside this backup and is not modified.

## Scope

The backup script is [`scripts/backup_internal_coordinator_do3.py`](../scripts/backup_internal_coordinator_do3.py). Run it as root on do3. New snapshots are encrypted before a single `.tar.gz.age` file is published beneath `/opt/rawrarr-internal-do3/backups`. The script requires at least 512 MiB free on both the backup filesystem and `/run` tmpfs, and refuses paths that grant group/other access.

## Personal age recipient

The recipient is the environment owner's existing personal age public recipient, separate from any Rawrarr node or group recipient. Provision it through the authenticated coordinator action described in the [owner-recipient continuation runbook](2026-11-03-environment-owner-backup-recipient.md). That action fetches only the derived public `age1pq1…` recipient from an online, account-owned fossil and pins it alongside the owner's account ID in PostgreSQL. The private identity stays on the fossil. No recipient file needs to be installed on do3, and `--recipient-file` is no longer supported. Never use `/shared/age-recipient`, `/shared/group-age-recipient`, a device recipient, or an ML-DSA signing-root key for this purpose.

The script reads the pin with a read-only query against only the internal coordinator database. A missing migration, absent owner pin, database failure, or invalid recipient stops the backup before collection. Once pinned, the recipient remains usable when fossils are offline or the source node has been removed. Re-provisioning with another recipient fails without changing the existing pin; ownership transfer and recipient rotation require a separate recovery procedure.

The script validates the recipient, stages plaintext only under `/run` tmpfs, streams a compressed tar archive through the `age` binary bundled in the running internal roarlay image, and writes ciphertext atomically to disk. The manifest is inside the encrypted archive and records only a SHA-256 fingerprint of the public recipient. No plaintext snapshot directory or tar file is left on disk by a new run. Age v1.3.2 in the Rawrarr image supports the project's post-quantum recipients. The private identity never enters the server or container.

Each backup contains:

- A custom-format, transaction-consistent `pg_dump` for the Rawrarr `rawrarr` database and the Umami `umami` database. The script checks each archive with `pg_restore --list` and saves the catalog.
- A compressed copy of the internal Tailscale state volume and the coordinator `/ui` volume. These are captured live; restore only with the related services stopped.
- The internal project Compose files and internal analytics Caddyfile.
- Every regular file in `/opt/rawrarr-internal-do3/secrets`, plus the mounted UI updater token from `/root/gh-token`, staged in tmpfs and stored only inside the encrypted archive.
- A Tailscale Serve status snapshot, container image IDs/tags/revision labels, service health metadata, and SHA-256 checksums.

The script does not copy Docker images, PostgreSQL live data directories, the public project, public Caddy/Headscale/Stripe state, or subscriber `/shared`, `/media`, and `/data`. Rebuild/pull the recorded published Rawrarr revision and pinned third-party images during recovery. The ciphertext remains on do3; copy it to a separately protected off-host destination if disaster recovery must survive do3 loss.

## Run and verify

Before running, check the internal services and available space; do not stop or recreate containers. The database dumps are made online through PostgreSQL's consistent logical dump mechanism. Tailscale and UI volumes are archived live without changing the running services.

```sh
tailscale ssh root@rawrarr-roarlay.risk-mermaid.ts.net 'docker context show; df -h /opt/rawrarr-internal-do3/backups'
tailscale ssh root@rawrarr-roarlay.risk-mermaid.ts.net 'python3 -' < scripts/backup_internal_coordinator_do3.py
```

Run both commands from the Rawrarr checkout containing the reviewed script, only after backup execution is separately authorized. The backup command refuses to run until the environment owner recipient is provisioned in the internal database. It prints the encrypted archive path and a decryption reminder, not the recipient or private identity. Verify the ciphertext decryption and the inner `SHA256SUMS` on Will's personal device, where the matching private ID stays:

```sh
age --decrypt --identity /path/to/personal-age-identity /path/to/internal-coordinator-<UTC timestamp>.tar.gz.age | tar -tzf - >/dev/null
```

For full content verification, decrypt into a `0700` directory on the personal device, run `sha256sum --check SHA256SUMS`, inspect both PostgreSQL archives with `pg_restore --list`, inspect both volume archives with `tar -tzf`, then remove the plaintext staging copy. Do not decrypt on do3. Never copy backup contents into the repository or print secret contents.

For the already-created plaintext snapshot, first provision the environment owner public recipient, then create an encrypted copy without modifying the original:

```sh
tailscale ssh root@rawrarr-roarlay.risk-mermaid.ts.net 'python3 - --encrypt-existing /opt/rawrarr-internal-do3/backups/internal-coordinator-20261028T104535Z' < scripts/backup_internal_coordinator_do3.py
```

Keep the original protected until Will verifies the encrypted copy on his personal device. Removing plaintext snapshots is a separate owner decision.

## Recovery

Recovery is a controlled restore, not a routine restart. Preserve the original volumes and the backup directory. Confirm the target is the internal Compose project before stopping anything. Restore the matching published Rawrarr revision and internal analytics image; the latter is the custom Umami 3.4.1 image built from commit `559e3c607c23d2c72d9bd4f6481bda428255323c` with `BASE_PATH=/analytics` (see the [internal analytics runbook](2026-10-28-internal-analytics-dashboard.md)). Stop only the internal services (`analytics-caddy`, `umami`, `roarlay`, `tailscale`, and both internal PostgreSQL services) after claiming their deployment locks. Recreate empty internal PostgreSQL databases and restore `coordinator.dump` and `analytics.dump` with `pg_restore`; restore the Tailscale state and UI archives into their matching named volumes while their consumers are stopped; restore the protected secrets and Compose files to `/opt/rawrarr-internal-do3` with restrictive ownership/modes. Do not reuse the Tailscale identity concurrently with another running copy.

Start the internal project, verify both database services, roarlay and Umami health, read `tailscale serve status --json`, confirm `rawrarr.risk-mermaid.ts.net:443` proxies to `http://127.0.0.1:8081` (the internal Caddy front, which routes the coordinator health endpoint to roarlay), and verify `https://rawrarr.risk-mermaid.ts.net/health`. If any verification fails, stop only the internal project and preserve both the pre-restore volumes and the backup for recovery. The public project is not part of this procedure.

## Compatibility and adoption

The owner-recipient continuation adds embedded SQLx migration `0012_environment_owner.sql`, a dedicated fossil public endpoint, and an authenticated coordinator provisioning API. Existing deployments remain unprovisioned until the owner deliberately selects an upgraded online fossil; no key is inferred or generated. Upgrade coordinator and fossil backend images from a published, merged release before provisioning. Old fossils remain compatible for existing operations but cannot provide the new endpoint. The script has no old-file fallback. No synchronized files or existing node heartbeat protocol change. See the continuation runbook for adoption and rollback limits. Logical dumps are intended to restore into PostgreSQL 17 or a compatible newer version; restoring into an older major version is unsupported.

## Execution record

The following is a historical record, not verification of the new owner-recipient flow or current deployment state. The continuation has not deployed code, registered a live owner recipient, or made a production backup.

Completed 2026-10-28. The plaintext snapshot is `/opt/rawrarr-internal-do3/backups/internal-coordinator-20261028T104535Z` (about 2.4 MiB). All 18 listed payload files passed `sha256sum --check`; both custom-format PostgreSQL archives passed `pg_restore --list` inside their respective PostgreSQL containers; both volume archives passed `tar -tzf`; required secrets including the mounted UI updater token are present; all files are mode `0600` and directories mode `0700`. The internal HTTPS health route returned the Roarlay health page. `tailscale serve status --json` confirmed `rawrarr.risk-mermaid.ts.net:443` proxies to `http://127.0.0.1:8081`. No containers were stopped or recreated. A first backup created at `10:45:03Z` is preserved but superseded because it predates inclusion of the mounted UI updater token; use the `10:45:35Z` snapshot. Age-encrypted output is not yet created because the personal operator public recipient has not been supplied or installed; the node/group recipient is intentionally not substituted.
