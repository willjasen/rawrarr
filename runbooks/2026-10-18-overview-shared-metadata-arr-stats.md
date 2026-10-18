# Overview shared metadata and Arr statistics

## Objective

Show shared-folder health, encrypted shared metadata size, and Radarr and Sonarr library totals on the coordinator Overview page.

## Initial state

The coordinator Overview contained only its heading. Device cards already fetched Syncthing shared-folder health; the metadata node exposed encrypted metadata byte totals; Movies and TV already loaded Radarr and Sonarr libraries.

## Actions

- Added responsive Overview cards for the shared folder, metadata size, Radarr, and Sonarr.
- Reused the existing shared-folder health cache, metadata storage endpoint, and media library loaders. Folder state is summarized with sync completion; metadata is split into movie and other bytes; Arr cards show item/file or episode/disk totals.
- Added setup and unavailable states for missing nodes, connections, browser identity, or service responses.
- Added Radarr and Sonarr genre breakdowns. Genre totals count each library entry once per distinct genre; Radarr genre data travels in the encrypted browser library response, and Sonarr includes genres in its normalized series response.

## Verification

`node --check` for the coordinator and metadata export modules and scoped `git diff --check` passed. An earlier build attempt exposed pre-existing Rust errors, which were fixed in the concurrently updated worktree before this deployment. Building with only the base Compose file then collided on port 7298; recreating roarlay with `docker-compose.yml`, `docker-compose.coordinator.yml`, and `docker-compose.tailscale-macbook.yml` succeeded. `rawrarr-local-roarlay-1` is healthy; local `/health` returns HTTP 200. Tailscale Serve still maps `rawrarr.risk-mermaid.ts.net:443` `/` to `http://127.0.0.1:8080`, and tailnet `/health` returns HTTP 200 identifying Roarlay. The authenticated Overview was not browser-checked.

## Recovery

Revert this runbook, its index entry, and the Overview markup and loader changes in `coordinator_workspace.html` and `coordinator-workspace.mjs`. To deploy, first resolve the unrelated Rust compile errors in the current worktree, then rebuild roarlay with the project deployment lock and verify local and tailnet health.
