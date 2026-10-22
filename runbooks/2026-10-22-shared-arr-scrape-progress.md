# Show shared Arr scraping progress on Movies and TV

## Objective and initial state

Make active Radarr and Sonarr metadata/artwork scraping clear on their coordinator library pages with a visible status and progress bar. The TV view already polled the Sonarr status endpoint. The Movies view did not expose the corresponding Radarr status, even though the metadata node provided `/metadata/artwork/backfill`.

## Actions

- Added Radarr status and progress elements alongside the Movies library.
- Consolidated both providers through `refreshArrScrapeStatus(provider)`, polling the existing provider-specific endpoints and displaying saved/total, checked-this-pass, and unavailable counts.
- Refreshes status on fossil selection for the active library and polls the active view every three seconds.
- Added source regression coverage and updated the script test index.

## Verification

- `node --test scripts/tests/test_rawrarr_sonarr_completion_ui.mjs` passed (2 tests).
- `git diff --check` passed.
- Committed as `c9be25d` and pushed to `origin/main`.
- Backed up the managed coordinator workspace assets under `~/Library/Application Support/rawrarr/internal-coordinator/backups/pre-c9be25d-shared-arr-progress/`, then applied only the task patch to the existing Mac mini build source. This preserved the mini's newer unrelated source changes.
- Rebuilt only `roarlay` in Compose project `rawrarr-internal-mini`. The first attempt stopped before container changes because Docker's keychain helper was missing from the noninteractive `PATH`; adding `/usr/local/bin` resolved it. PostgreSQL and the Tailscale sidecar were not restarted.
- Roarlay is healthy, PostgreSQL remains healthy, and Tailscale Serve JSON maps `rawrarr.risk-mermaid.ts.net:443` `/` to `http://127.0.0.1:8080`. `https://rawrarr.risk-mermaid.ts.net/health` returned HTTP 200.
- Refreshed the authenticated Chrome Movies page. The selected fossil library rendered 1,945 entries. Scraping was not active during read-back, so the progress bar was hidden as designed.
- Removed the roarlay deployment lock and confirmed it was absent.

## Recovery and remaining work

To roll back the live coordinator assets, restore `coordinator_workspace.html` and `coordinator-workspace.mjs` from the backup directory above, rebuild only roarlay under its deployment lock, and repeat the health and Serve checks. No database or Tailscale state changed.
