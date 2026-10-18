# Rescan metadata from Movies and TV

## Objective and initial state

Change the Movies and TV page actions from refresh to Rescan and have each action request a metadata rescan from the selected metadata node. The checkout already contained extensive work from other tasks, including edits to the coordinator workspace and metadata service; preserve all unrelated changes and leave overlapping files uncommitted.

Related: [Browse the Sonarr library on TV](2026-10-18-browse-sonarr-library.md), [Automatically backfill Sonarr metadata and artwork](2026-10-18-automatic-sonarr-metadata-backfill.md), and [Complete Radarr and Sonarr text metadata](2026-10-18-complete-arr-text-metadata.md).

## Actions

- Added owner-session-protected POST endpoints on metadata nodes to signal their existing Radarr and Sonarr metadata workers.
- Movie rescan requests refresh stored Radarr text metadata, including entries with existing artwork. TV rescan requests wake the Sonarr metadata worker, which re-reads series and episode data and updates changed encrypted records.
- Changed the Movies and TV buttons to “Rescan”; each action contacts the currently selected metadata node directly and reports that its worker started.

## Verification

- `node --check rust-arr/src/coordinator-workspace.mjs` and `git diff --check` passed.
- The host `cargo check --locked --offline` could not start because Homebrew LLVM could not load `libz3.5.1.dylib`. The repository Compose build compiled the Rust application successfully; it reported one existing dead-code warning for `read_shared_recipient`.
- Rebuilt and recreated only the `metadata` and `roarlay` services in Compose project `rawrarr-local`, using the Compose file set from their existing labels and passing the protected PostgreSQL password-file path without reading its contents. Both containers became healthy, and the metadata `/health` endpoint returned HTTP 200.
- Roarlay local health and `https://rawrarr.risk-mermaid.ts.net/health` returned the page identifying Roarlay. Tailscale Serve read-back confirms HTTPS 443 `/` proxies to `http://127.0.0.1:8080`; the existing `/direct-api` handler remains configured.
- Per-service deployment markers were removed after verification. No live metadata rescan was initiated during implementation.
- The repository had extensive pre-existing changes, including changes in all source files touched here and the runbook index; task changes remain uncommitted to avoid staging overlapping work.

## Recovery

Revert only the rescan route/worker signal changes and the two page button handlers/labels. Existing encrypted metadata records remain intact.
