# Show shared Arr scraping progress on Movies and TV

## Objective and initial state

Make active Radarr and Sonarr metadata/artwork scraping clear on their coordinator library pages with a visible status and progress bar. The TV view already polled the Sonarr status endpoint. The Movies view did not expose the corresponding Radarr status, even though the metadata node provided `/metadata/artwork/backfill`.

## Actions

- Added Radarr status and progress elements alongside the Movies library.
- Consolidated both providers through `refreshArrScrapeStatus(provider)`, polling the existing provider-specific endpoints and displaying saved/total, checked-this-pass, and unavailable counts.
- Refreshes status on fossil selection for the active library and polls the active view every three seconds.
- Added source regression coverage and updated the script test index.

## Verification

- `node --test scripts/tests/test_rawrarr_sonarr_completion_ui.mjs` — pending.
- `git diff --check` — pending.
- Deployment to the internal coordinator has not been performed.

## Recovery and remaining work

Revert only this task's coordinator workspace UI/module, focused test/index, and this runbook/index entry to restore the previous progress display behavior. Deploy only when explicitly requested and follow coordinator deployment and Serve verification instructions.
