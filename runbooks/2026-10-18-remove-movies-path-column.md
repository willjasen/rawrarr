# Remove Movies Path column

## Objective and initial state

Remove the Path column from the Movies page. The worktree contains unrelated changes, which are preserved. Port 7297 is owned by `rawrarr-local-rawrarr-1`.

Related: [Movies table text](2026-10-18-larger-movie-table-text.md), [responsive movie refresh](2026-10-18-responsive-movies-refresh.md).

## Actions

Removed the Path heading and corresponding row cell. Updated the existing Chromium column-alignment test to verify paths are absent even when provided by the API.

## Verification

- Seven focused movie Chromium/Node tests passed (identifiers and column alignment, links, responsiveness).
- `git diff --check` passed.
- Rebuilt the existing `rawrarr-local` app on port 7297, preserving data and configuration; container status is healthy and `/health` returned HTTP 200.
- Authenticated live Movies page was not inspected; browser verification used controlled API fixtures.

## Final state

Path column removed and deployed. Only task-owned changes staged, including isolated edits to already-modified files. Pre-existing modifications and untracked files remain. No remote is configured, so the task commit stays local.

## Recovery

Restore only the removed heading and row value, then rebuild the existing `rawrarr-local` app service with its existing volumes.
