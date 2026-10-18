# Show Sonarr series artwork on TV

## Objective

Display Sonarr series artwork in the coordinator TV library, matching the poster-and-title layout used by Movies.

## Initial state

The Sonarr scraper had already saved eligible posters under the age-encrypted TV artwork tree, but the coordinator TV table rendered titles as plain text. The worktree contains extensive unrelated edits; preserve them.

Related procedures: [Sonarr artwork backfill](2026-10-18-automatic-sonarr-metadata-backfill.md), [Movies artwork](2026-10-18-radarr-movie-artwork.md).

## Actions

- Added a TVDB-keyed encrypted artwork reader and an authenticated poster endpoint on both the regular and direct metadata APIs.
- Added lazy poster images beside TV series titles. URLs use the selected metadata node's direct API route; missing artwork leaves the title visible.
- Added a regression assertion for the poster route, lazy rendering, and age-vault read.

## Verification

- Focused coordinator/Sonarr tests passed (17/17); the age-encrypted TV poster round-trip test passed. JavaScript syntax and `git diff --check` passed.
- The full isolated runner stalled in an existing Chromium settings flow and was stopped; the focused tests completed successfully when run separately.
- Rebuilt metadata and roarlay in `rawrarr-local` under separate service locks. Metadata is healthy, and unauthenticated access to the new direct poster route returns HTTP 401 as expected.
- Roarlay is healthy; local and HTTPS health checks identify Roarlay. Tailscale Serve still maps `/` to `http://127.0.0.1:8080` and `/direct-api` to `http://127.0.0.1:8082`.
- The signed-in live `#tv` view loads 228 series and displays cached poster thumbnails beside titles. Browser checks confirmed visible poster images loaded from the direct metadata route.
- The working tree already contained extensive unrelated changes before this task. Mixed source files and pre-existing untracked files remain uncommitted; no push was attempted.

## Recovery

Revert the endpoint, encrypted-artwork reader, and TV poster rendering to restore the text-only series table. Existing shared ciphertext can remain in place.
