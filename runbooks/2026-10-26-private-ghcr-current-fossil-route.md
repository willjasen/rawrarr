# Private GHCR subscriber images and current Mac mini route

Issue: [#176](https://github.com/willjasen/rawrarr/issues/176)

## Goal

Make the generated fossil/boneyard Compose file pull the Rawrarr image from private GitHub Container Registry and route the re-enrolled Mac mini fossil through the coordinator.

## Changes

- GitHub Actions publishes `ghcr.io/willjasen/rawrarr` for Linux amd64 and arm64 on relevant `main` pushes. It publishes rolling `main` and immutable commit-SHA tags. The newly published package defaults to private; maintainers must preserve that visibility unless the owner chooses to make it public.
- Generated subscriber Compose references `ghcr.io/willjasen/rawrarr:main`. Before the first pull, a user with repository package access must log in to `ghcr.io` using a GitHub token with `read:packages`, then run `docker compose pull` and `docker compose up -d`. No registry credential is written into Compose.
- Internal Mac mini coordinator relay/direct maps use current fossil node ID `device-8de78fadda6400a45ca1a30b` at `https://rawrarr-mac-mini.risk-mermaid.ts.net/` and its `/direct-api/` endpoint. This lets coordinator relay and browser direct-route discovery resolve the same enrollment ID.

## Verification

- Run `node --test scripts/tests/test_rawrarr_subscriber_compose.mjs`.
- Run `node --test scripts/tests/test_rawrarr_coordinator_workspace.mjs`.
- Validate the workflow syntax and target architectures; confirm GHCR package visibility and a successful multi-architecture published digest after merge.
- After deployment from published `main`, verify roarlay health and the tailnet HTTPS route using the coordinator deployment procedure in `AGENTS.md`. Confirm the existing fossil record is online and its node avatar loads.

## Operational note

This image change only applies to newly generated Compose files. Existing fossil containers can keep running their current image. Private package pulls require an authorized user's GHCR login; an unauthenticated user cannot pull it while package visibility remains private.
