# Separate movie identifiers from titles

## Objective and initial state
Move the identifier appended to Movies table titles into its own column. Inspection confirmed these are TMDb IDs, not IMDb IDs. The UI and indexes already contain substantial unrelated changes. Port 7297 belongs to the healthy `rawrarr-local-rawrarr-1` container.

Related: [movie links](2026-10-18-radarr-movie-links.md), [larger table text](2026-10-18-larger-movie-table-text.md).

## Actions
- Render only the title in the movie title span; display the existing identifier in a separate TMDb ID column, with an em dash when absent.
- Preserve poster/title links and the remaining column alignment.
- Add browser coverage for populated and missing IDs and literal title rendering.

## Verification
- Eight focused tests passed in the isolated Chromium test image: movie identifiers, movie links, and artwork source.
- `git diff --check` passed.
- Rebuilt the existing `rawrarr-local` web service using the documented protected password-file path without reading its contents. `/health` returned HTTP 200; the container reports healthy after startup.
- Browser behavior was checked with isolated fixtures; the authenticated live library was not opened.

## Git state
Task-owned tests, runbook, and documentation additions are committed on main. UI edits remain uncommitted because their surrounding movie-rendering implementation belongs to pre-existing uncommitted work. Other pre-existing modified and untracked files remain intact. No remote or upstream is configured, so no push is possible.

## Recovery
Undo only this task's UI changes and rebuild the existing Compose service, preserving configuration and volumes.
