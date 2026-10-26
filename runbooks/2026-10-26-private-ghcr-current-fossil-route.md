# Private GHCR subscriber images and current Mac mini route

Issue: [#176](https://github.com/willjasen/rawrarr/issues/176)

## Goal

Make the generated fossil/boneyard Compose file pull the Rawrarr image from private GitHub Container Registry and route the re-enrolled Mac mini fossil through the coordinator.

## Changes

- GitHub Actions builds Linux amd64 and arm64 as separate matrix jobs and pushes unique commit-and-architecture tags. A dependent manifest job publishes rolling `main` and immutable commit-SHA tags only after both architecture builds succeed. The newly published package defaults to private; maintainers must preserve that visibility unless the owner chooses to make it public.
- Generated subscriber Compose references `ghcr.io/willjasen/rawrarr:main`. Before the first pull, a user with repository package access must log in to `ghcr.io` using a GitHub token with `read:packages`, then run `docker compose pull` and `docker compose up -d`. No registry credential is written into Compose.
- Internal Mac mini coordinator relay/direct maps use current fossil node ID `device-8de78fadda6400a45ca1a30b` at `https://rawrarr-mac-mini.risk-mermaid.ts.net/` and its `/direct-api/` endpoint. This lets coordinator relay and browser direct-route discovery resolve the same enrollment ID.

## Deployment split

Deploy coordinator and fossil as independent operations, never with a project-wide `docker compose up`:

1. Atomically claim `.container-rebuild-roarlay.lock`; recreate only `roarlay` in `rawrarr-internal-mini`. Verify roarlay health and the existing HTTPS Serve route, then release and confirm the lock is gone.
2. Separately atomically claim `.container-rebuild-fossil.lock`; recreate only `fossil` in its existing Compose project. Verify fossil health and its route, then release and confirm the lock is gone.

Each operation uses the exact merged `main` revision, preserves PostgreSQL/Tailscale and named volumes, and has its own rollback image. Do not hold or release both locks as one combined deployment. Current live inspection found the coordinator and fossil in separate projects on the Mac mini; the MacBook currently has no fossil container. Re-check live project placement and service status immediately before each operation.

## Verification

- Run `node --test scripts/tests/test_rawrarr_subscriber_compose.mjs`.
- Run `node --test scripts/tests/test_rawrarr_coordinator_workspace.mjs`.
- Run `python3 scripts/tests/test_publish_container_workflow.py` and validate workflow YAML syntax.
- Confirm the AMD64 and ARM64 jobs pass separately and the manifest job publishes the multi-architecture tags; confirm GHCR package visibility remains private.
- After deployment from published `main`, verify roarlay health and the tailnet HTTPS route using the coordinator deployment procedure in `AGENTS.md`. Confirm the existing fossil record is online and its node avatar loads.

## Operational note

This image change only applies to newly generated Compose files. Existing fossil containers can keep running their current image. Private package pulls require an authorized user's GHCR login; an unauthenticated user cannot pull it while package visibility remains private.
