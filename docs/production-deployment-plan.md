# Production deployment plan

## Decision for the first release

Start with a self-hosted PostgreSQL instance on the coordinator host, deployed as a dedicated Compose service with a persistent volume and protected password file. Keep it private to the Compose network; do not publish port 5432. This matches the current coordinator design and keeps the first deployment operationally simple. Treat the database as critical state: passkeys, sessions/ceremonies, approved nodes, and account ownership live there. It is separate from subscriber `/shared`, `/data`, and media.

This is a provisional hosting choice, not a commitment to keep Postgres on that host forever. Revisit managed Postgres (Neon is one candidate) when the service has real external users or when database availability, backups, patching, and operations become a meaningful burden. Popularity alone is not the trigger; use operational needs and measured load.

## Before the first production launch

1. Choose and secure the coordinator host. Restrict administration, apply OS/container updates, configure firewall rules, and provide HTTPS at the public or private hostname. Never expose PostgreSQL publicly. The coordinator must not mount subscriber `/shared`, `/media`, private age identities, or subscriber app-data.
2. Configure the coordinator’s exact HTTPS origin and WebAuthn RP ID before owners register passkeys. Passkeys are scoped to their origin/RP; changing hostname later requires a deliberate migration/re-enrollment plan. Do not treat `localhost:7297` credentials as production credentials.
3. Deploy the coordinator-only Compose shape (`docker-compose.coordinator.yml`) with a pinned/reviewed application image, a strong password in a host-protected secret file, and the approved-node registry. Keep the DB volume on durable host storage. Subscriber nodes remain separate and connect over authenticated HTTPS.
4. Confirm SQL migrations run on startup against an empty production database, then create the first owner account and validate sign-in, session expiry, node enrollment/approval, revocation, and restart persistence. Current `/health` only reports web-process liveness; add/readiness monitoring that actually exercises database connectivity before relying on health checks for operations.
5. Set up encrypted/off-host database backups before storing real accounts. Define retention, access control, restore procedure, and recovery-point expectations. Test a restore into an isolated database and verify passkeys, account ownership, approved-node state, and authentication. Do not include PostgreSQL datadirs in Syncthing or `/shared` backups.
6. Establish monitoring for coordinator and database availability, disk capacity, backup freshness, migration failures, and authentication errors. Document who can access production secrets and how to rotate the DB credential without logging it.
7. Roll out a small private pilot first. Keep a known-good app image and a tested database backup before upgrades. Prefer forward-compatible migrations and roll forward when possible; do not assume an older binary can start after a newer migration.

## Local Postgres versus Neon

### Initial self-hosted Postgres

Use the existing Compose pattern as the production starting point: a private Postgres service, durable named/bind volume, password file mounted as a secret, automated backups copied off-host, and coordinator-only network access. Compose gives control and avoids adding a managed service dependency, but the operator owns host uptime, storage, upgrades, monitoring, backups, and restore testing. A single host/database remains a single failure domain.

### Future managed Postgres (Neon or equivalent)

The coordinator already supports `RAWARR_DATABASE_URL`; `RAWARR_PG_*` plus a password file remains the local Compose path. A future managed deployment can use the URL path, but first verify the provider’s TLS requirements, connection-string format, pooling behavior, connection limits, and SQLx compatibility. The application currently allows up to 10 pooled connections per coordinator process; size provider limits with that cap and the number of replicas in mind. Store the URL in the deployment secret manager, never in Compose source, docs, logs, or Git.

Before switching providers, test migrations and WebAuthn flows on a disposable Neon project, configure provider-side backups/restore guarantees, and rehearse cutover. Freeze authentication writes, take and verify a logical backup, restore/import into Neon, point a staging coordinator at it, validate owner/passkey and node state, then cut over the coordinator using the new secret. Keep the old database untouched and recoverable until production sign-in and node authentication are verified. Define a rollback that restores the old connection target and accounts for writes made after cutover; avoid split-brain writes.

## Reassessment triggers

Review the hosting choice when any of these occurs: external users depend on the service; the coordinator needs multi-host failover; backup/restore or patching is not consistently exercised; database downtime exceeds the chosen recovery objective; database capacity/connection limits become visible; or operating Postgres takes more time than managed-service cost justifies. Compare total cost and operational fit rather than assuming traffic volume alone requires Neon.

## Current implementation notes and open work

- `docker-compose.coordinator.yml` currently contains the private Postgres + coordinator shape and a named DB volume.
- `rust-arr/src/auth.rs` supports `RAWARR_DATABASE_URL` and the discrete `RAWARR_PG_*` connection fields. The pool limit is 10 connections.
- `docker-compose.yml` is the persistent local development/self-hosting deployment, not a complete hardened production recipe.
- Production readiness still needs a selected host/domain/TLS approach, pinned release image and upgrade procedure, DB-aware readiness, off-host backup/restore validation, and an explicit WebAuthn origin/RP decision.

Related: [container deployment](container-deployment.md), [data domains](data-domains.md), and [cloud access architecture](cloud-access-architecture.md).
