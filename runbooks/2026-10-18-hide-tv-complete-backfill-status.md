# Hide TV backfill status when posters are saved

## Objective and initial state

The TV page kept showing an active Sonarr backfill message such as “226 / 226 posters saved” while the worker checked the library for additional series. The UI required both all posters saved and the worker inactive before hiding completion details.

## Changes

- Treat the artwork cache as complete whenever `scraped >= total`, regardless of the worker's active check pass.
- Keep the status visible when eligible posters are still missing.
- Added a regression assertion for a fully saved cache with an active worker.

## Verification

- `node --test scripts/tests/test_rawrarr_sonarr_completion_ui.mjs` passed (1 test).
- `git diff --check` passed.
- Rebuilt roarlay in the existing `rawrarr-local` Compose project. It reports healthy; local and tailnet health endpoints return HTTP 200.
- Tailscale Serve read-back confirms HTTPS `/` still proxies to `http://127.0.0.1:8080` and `/direct-api` remains on `http://127.0.0.1:8082`.
- The UI now hides the progress and successful-load messages once saved posters meet the total, even if the worker is active checking for new series. Browser-visible behavior was not read back in this deployment.

## Final state

The completion condition is deployed to roarlay. Changes remain local and uncommitted because the worktree contains extensive pre-existing changes.
