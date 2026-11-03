# Restore Rawrarr on do3

## Objective and issue

Stop the Synkpieces environment on the Rawrarr droplet and start Rawrarr from published `main`. After discovering that the old database's migration history is incompatible, Will authorized a fresh current-main database while keeping the old database volume and backup intact. Tracked by [issue #199](https://github.com/willjasen/rawrarr/issues/199).

## Target and starting state

- Refreshed `origin/main` and confirmed the task worktree is based on `fc229749cecd47a2282a75719a0cdb9e9e801fce`.
- The documented Tailscale SSH endpoint `100.91.193.80` identified itself as `rawrarr-roarlay`, Debian 13, amd64. The earlier Synkpieces deployment record maps this host to DigitalOcean address `104.248.184.91`.
- Initially, Rawrarr Compose projects `rawrarr-roarlay-prod` and `rawrarr-subscribers` were stopped and preserved. Synkpieces Compose project `synkpieces-coordinator` was running.
- The Rawrarr PostgreSQL database is populated. Aggregate counts were one user, three passkeys, one node, and one enrollment. Its migration history contains versions 1–10, including a different branch-specific migration sequence after version 5.

## Actions

- Stopped `synkpieces-coordinator` with `docker compose stop`. Stopped its separately profiled `cloudflared` container as well. Kept all Synkpieces containers, its two named Postgres volumes, the tunnel resource, and DNS configuration. No `down` or volume removal was run.
- Started only the existing Rawrarr PostgreSQL container to make a fresh backup. The first dump attempt used the nonexistent default database role `postgres` and created a zero-byte task-owned file; removed that file and retried using the configured `rawrarr` role.
- Created `/opt/rawrarr-roarlay-prod/backups/pre-fc22974-20261027.dump` on the droplet with mode 600. `pg_restore --list` succeeded; the custom-format dump is 38,657 bytes.
- Confirmed the GitHub Tests workflow passed for `fc229749cecd47a2282a75719a0cdb9e9e801fce`. Pulled the immutable amd64 image tag from GHCR and verified its OCI revision label is the same full commit and its platform is `linux/amd64`.
- Added a host-local Compose override pointing roarlay at that immutable image. The first startup against the old database exited with SQLx `VersionMissing(9)` because the migration chain differs. Stopped roarlay without changing the existing database.
- After Will selected a clean current-main database, added a second host-local Compose override that mounts a new named volume, `rawrarr-roarlay-prod_rawrarr-roarlay-prod-postgres-data-main`, at PostgreSQL's data path. Started roarlay and its declared dependencies (PostgreSQL, Headscale, Caddy, and Tailscale). The old volume `rawrarr-roarlay-prod-postgres-data` remains present, unattached, and unchanged; its verified pre-deployment dump remains at `/opt/rawrarr-roarlay-prod/backups/pre-fc22974-20261027.dump`.

## Current state and database implications

- Synkpieces containers, including `cloudflared`, are stopped. Its named volumes remain present. Its public tunnel and DNS record were preserved.
- Rawrarr PostgreSQL and roarlay are healthy and running. Caddy, Tailscale, and the primary Headscale container are running with their existing configuration and state. The new database has migrations 1–8 and zero users, passkeys, or nodes. The old Swarmsail and subscriber-provisioner workers and the separate `rawrarr-subscribers` project remain stopped; they were not required for the current coordinator launch and were not part of this recovery.
- Current `main` has migrations 1–8: `0006_revoked_nodes`, `0007_ai_requests`, and `0008_instance_device_signing` follow version 5. The database records migrations 1–10 from a different source history: version 6 is `ai requests`, 7 is `node bootstrap tokens`, 8 is `account tailnets`, 9 is `browser access authorization`, and 10 is `browser rendezvous`.
- The prior image's source commit is not available from GitHub. Current `main` cannot safely migrate the old database automatically. Its volume and backup are retained for recovery/reference; the fresh database does not contain the previous account, passkeys, or node state.

## Verification and remaining work

- Verified target hostname and OS over Tailscale SSH; verified the exact image revision/platform from its OCI labels (`fc229749cecd47a2282a75719a0cdb9e9e801fce`, `linux/amd64`).
- Verified the backup with `pg_restore --list`; the old Rawrarr and Synkpieces named volumes remain present. Verified the new database's migrations 1–8 and zero users/passkeys/nodes.
- Verified roarlay and Postgres report healthy, the app log says the coordinator is listening on port 8080, and TLS-verified `curl -fsS https://rawrarr.willjasen.net/health` returned HTTP 200 with the Roarlay health page.
- The clean database requires initial owner setup in the browser: create the new account/passkey, save the new age/signing root material and recovery backup, then re-enroll nodes. The previous account and nodes remain only in the preserved old volume/dump and are not active on this fresh trust domain.
- Do not remove the old volume, dump, or Synkpieces data as part of later cleanup without a separate explicit request.
