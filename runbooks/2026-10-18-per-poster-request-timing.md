# Per-poster request timing

## Objective and initial state
Correct Movies load times that include time waiting to scroll. The existing lazy image timer starts at row creation. The working tree already contains extensive unrelated changes. Port 7297 belongs to healthy `rawrarr-local-metadata-1`; the current Compose service is `metadata`, replacing the older `rawrarr` name.

Prior context: [original timing](2026-10-18-radarr-artwork-load-time.md), [artwork source](2026-10-18-show-movie-artwork-source.md).

## Actions
Use browser Resource Timing requestStart through responseEnd per image URL, collected by PerformanceObserver so large libraries do not overflow the resource buffer. Exclude lazy wait, browser queueing and decode time. Show Pending before load, Failed on error, and Unavailable if timing is absent or restricted. Shared renderer applies the correction to series too.

## Verification and final state
- Six Node assertions and four Chromium checks passed, including a delayed scroll to poster 150 that would fail with the original cumulative timer. `git diff --check` passed.
- First rebuild failed on concurrent node-authentication edits; those were corrected by concurrent work without edits from this task. Retried successfully with `RAWARR_POSTGRES_PASSWORD_FILE` pointing to the existing protected file and `docker compose -p rawrarr-local up -d --no-deps --build metadata`.
- The metadata container is healthy and local `/health` returns HTTP 200. Existing data volumes and other services preserved. No private live-library session was inspected; browser verification used sanitized fixtures.
- UI and existing artwork-source test changes depend on substantial pre-existing uncommitted work (the original timing feature itself is uncommitted). Leave those mixed files uncommitted; commit only new tests, this runbook, and task-owned documentation additions. Branch `main` has no configured upstream, so no push.

## Recovery
Reverse only this task's UI changes and rebuild the existing metadata service with the same Compose project and volumes.
