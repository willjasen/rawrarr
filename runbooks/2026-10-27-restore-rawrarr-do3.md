# Restore Rawrarr on do3

## Objective and issue

Stop the Synkpieces environment on the Rawrarr droplet and bring Rawrarr back on the latest published `main` image without losing the existing account, passkeys, node state, databases, or network identities. Tracked by [issue #199](https://github.com/willjasen/rawrarr/issues/199).

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
- Added a host-local Compose override pointing roarlay at that immutable image. Started roarlay and its declared dependencies (PostgreSQL, Headscale, Caddy, and Tailscale). The current coordinator exited during startup with SQLx `VersionMissing(9)` because the existing database has a different migration chain. Stopped the failing roarlay container without restoring or changing the database.

## Current state and blocker

- Synkpieces containers, including `cloudflared`, are stopped. Its named volumes remain present. Its public tunnel and DNS record were preserved.
- Rawrarr PostgreSQL is healthy and running. Caddy, Tailscale, and the primary Headscale container are running with their existing configuration and state. The new roarlay container is stopped after the migration failure. The old Swarmsail and subscriber-provisioner workers and the separate `rawrarr-subscribers` project remain stopped.
- Current `main` has migrations 1–8: `0006_revoked_nodes`, `0007_ai_requests`, and `0008_instance_device_signing` follow version 5. The database records migrations 1–10 from a different source history: version 6 is `ai requests`, 7 is `node bootstrap tokens`, 8 is `account tailnets`, 9 is `browser access authorization`, and 10 is `browser rendezvous`.
- The prior image's source commit is not available from GitHub. Current `main` cannot safely migrate this database automatically. No migration rows were changed, and no account, passkey, or node data was reset.

## Verification and remaining work

- Verified target hostname and OS over Tailscale SSH; verified the exact image revision/platform from its OCI labels.
- Verified the database backup with `pg_restore --list` and the Synkpieces volumes remain present after stopping the stack.
- The verified public `https://rawrarr.willjasen.net/health` route returns HTTP 502 because the current `main` coordinator cannot start against the legacy migration history. The required next step is either a reviewed, data-preserving compatibility migration merged into `main`, or an owner-approved clean current-main bootstrap that leaves the existing database volume and backup intact but requires a new account/passkey/root setup and node re-enrollment.
- Do not edit `_sqlx_migrations`, restore or replace the existing database, or start a fresh trust domain until the owner selects the migration path.
