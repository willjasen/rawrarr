# Back up the internal coordinator on do3

## Objective and issue

Create and verify a protected, recoverable disk backup of the internal coordinator and its private analytics services. Tracked by [issue #299](https://github.com/willjasen/rawrarr/issues/299). The public `rawrarr-roarlay-prod` project is deliberately outside this backup and is not modified.

## Scope

The backup script is [`scripts/backup_internal_coordinator_do3.py`](../scripts/backup_internal_coordinator_do3.py). Run it as root on do3. It writes a unique directory beneath `/opt/rawrarr-internal-do3/backups`, refuses a destination with group/other access, requires at least 512 MiB free, and creates files with mode `0600` under a `0700` directory.

Each backup contains:

- A custom-format, transaction-consistent `pg_dump` for the Rawrarr `rawrarr` database and the Umami `umami` database. The script checks each archive with `pg_restore --list` and saves the catalog.
- A compressed copy of the internal Tailscale state volume and the coordinator `/ui` volume. These are captured live; restore only with the related services stopped.
- The internal project Compose files and internal analytics Caddyfile.
- Every regular file in `/opt/rawrarr-internal-do3/secrets`, plus the mounted UI updater token from `/root/gh-token`, copied into the root-only backup directory.
- A Tailscale Serve status snapshot, container image IDs/tags/revision labels, service health metadata, and SHA-256 checksums.

The script does not copy Docker images, PostgreSQL live data directories, the public project, public Caddy/Headscale/Stripe state, or subscriber `/shared`, `/media`, and `/data`. Rebuild/pull the recorded published Rawrarr revision and pinned third-party images during recovery. The backup remains on do3 and is not encrypted separately; root-only permissions protect it from other local accounts, but not from host/root compromise or disk loss. Keep a separately protected off-host copy if disaster recovery requires surviving do3 loss.

## Run and verify

Before running, check the internal services and available space; do not stop or recreate containers. The database dumps are made online through PostgreSQL's consistent logical dump mechanism. Tailscale and UI volumes are archived live without changing the running services.

```sh
tailscale ssh root@rawrarr-roarlay.risk-mermaid.ts.net 'docker context show; df -h /opt/rawrarr-internal-do3/backups'
tailscale ssh root@rawrarr-roarlay.risk-mermaid.ts.net 'python3 -' < scripts/backup_internal_coordinator_do3.py
```

Run both commands from the Rawrarr checkout containing the reviewed script. After creation, verify the reported backup directory and run on do3:

```sh
cd /opt/rawrarr-internal-do3/backups/internal-coordinator-<UTC timestamp>
sha256sum --check SHA256SUMS
docker exec -i rawrarr-internal-do3-postgres pg_restore --list < coordinator.dump >/dev/null
docker exec -i rawrarr-internal-do3-umami-postgres-1 pg_restore --list < analytics.dump >/dev/null
tar -tzf tailscale-state.tar.gz >/dev/null
tar -tzf coordinator-ui.tar.gz >/dev/null
stat -c '%a %n' .. . secrets/* *.dump *.tar.gz
```

Record the backup directory, checksum result, file count/size, database catalog results, free space, and the internal HTTPS `/health` result in this runbook after execution. Never copy backup contents into the repository or print secret contents.

## Recovery

Recovery is a controlled restore, not a routine restart. Preserve the original volumes and the backup directory. Confirm the target is the internal Compose project before stopping anything. Restore the matching published Rawrarr revision and internal analytics image; the latter is the custom Umami 3.4.1 image built from commit `559e3c607c23d2c72d9bd4f6481bda428255323c` with `BASE_PATH=/analytics` (see the [internal analytics runbook](2026-10-28-internal-analytics-dashboard.md)). Stop only the internal services (`analytics-caddy`, `umami`, `roarlay`, `tailscale`, and both internal PostgreSQL services) after claiming their deployment locks. Recreate empty internal PostgreSQL databases and restore `coordinator.dump` and `analytics.dump` with `pg_restore`; restore the Tailscale state and UI archives into their matching named volumes while their consumers are stopped; restore the protected secrets and Compose files to `/opt/rawrarr-internal-do3` with restrictive ownership/modes. Do not reuse the Tailscale identity concurrently with another running copy.

Start the internal project, verify both database services, roarlay and Umami health, read `tailscale serve status --json`, confirm `rawrarr.risk-mermaid.ts.net:443` proxies to `http://127.0.0.1:8081` (the internal Caddy front, which routes the coordinator health endpoint to roarlay), and verify `https://rawrarr.risk-mermaid.ts.net/health`. If any verification fails, stop only the internal project and preserve both the pre-restore volumes and the backup for recovery. The public project is not part of this procedure.

## Compatibility and adoption

This is an operator backup procedure; it changes no database schema, synchronized files, node protocol, or running service configuration. Existing deployments adopt it by running the script on do3. No migration is required. Logical dumps are intended to restore into PostgreSQL 17 or a compatible newer version; restoring into an older major version is unsupported.

## Execution record

Completed 2026-10-28. The complete backup is `/opt/rawrarr-internal-do3/backups/internal-coordinator-20261028T104535Z` (about 2.4 MiB). All 18 listed payload files passed `sha256sum --check`; both custom-format PostgreSQL archives passed `pg_restore --list` inside their respective PostgreSQL containers; both volume archives passed `tar -tzf`; required secrets including the mounted UI updater token are present; all files are mode `0600` and directories mode `0700`. The internal HTTPS health route returned the Roarlay health page. `tailscale serve status --json` confirmed `rawrarr.risk-mermaid.ts.net:443` proxies to `http://127.0.0.1:8081`. No containers were stopped or recreated. A first backup created at `10:45:03Z` is preserved but superseded because it predates inclusion of the mounted UI updater token; use the `10:45:35Z` backup for recovery.
