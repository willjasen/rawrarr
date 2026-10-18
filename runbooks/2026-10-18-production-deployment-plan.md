# Plan the first production deployment

## Objective

Record a practical first-production database choice and deployment checklist, including a later managed Postgres migration path.

## Initial state

The repository has a coordinator-only Compose file with private PostgreSQL, a persistent volume, and a protected password-file secret. The coordinator supports both `RAWARR_DATABASE_URL` and discrete `RAWARR_PG_*` configuration. The working tree already contained extensive unrelated user changes; this task only added the production plan and updated documentation indexes.

## Decision and actions

Recommended initial production setup: self-host Postgres alongside the coordinator on its private Compose network, keep its port unpublished, use durable storage and a protected secret file, and implement off-host backups plus restore testing before real users. Keep Neon as a future option triggered by availability/operations/capacity needs rather than popularity alone.

Added [production deployment plan](../docs/production-deployment-plan.md), covering launch prerequisites, origin-bound passkeys, backup/monitoring, current database configuration, Neon connection requirements, migration rehearsal, cutover, and rollback. Linked it from the docs and runbook indexes.

## Verification

Read existing coordinator deployment docs, Compose definitions, database connection code, and indexes. Confirmed `RAWARR_DATABASE_URL` is accepted, the pool cap is 10 connections, and coordinator Compose keeps Postgres in the internal service network. Documentation-only; no deployment, migration, or database changes were performed.

## Final state and remaining work

Plan and indexes are updated. Before production, select the host/domain/TLS arrangement, finalize the WebAuthn RP/origin, ship a pinned release image, add DB-aware readiness, and establish and test off-host backup/restore. No commit was created because the repository instructions say to stop when a commit would leave no uncommitted changes; pre-existing user changes remain in the worktree.
