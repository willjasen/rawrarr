# Automatically backfill Sonarr metadata and artwork

## Objective

Scrape missing Sonarr series metadata and posters into age-encrypted shared metadata after first connection and periodically afterward, following the Radarr worker's sequential cache-first procedure.

## Initial state

The coordinator TV page displayed live Sonarr library fields, but the metadata node had no Sonarr scrape worker. Radarr already starts a metadata-node worker, skips cached artwork, stores encrypted metadata and posters, and repeats every six hours. The current repository worktree contains unrelated pre-existing edits; preserve them.

## Actions

- Added a metadata-role background worker that wakes after startup, waits for the encrypted config vault and Sonarr connection, then reads the Sonarr series list. Saving or clearing Sonarr settings wakes the worker; after each pass it waits six hours, matching Radarr's backfill cadence.
- The worker serially processes entries with a positive TVDB ID and a poster, skips cached text/artwork, writes metadata to `metadata/tv/text` and age-encrypted poster bytes to `metadata/tv/artwork`, and reports progress through an authenticated metadata endpoint. Series filesystem paths and API keys are not included in shared metadata.
- Followed Radarr's provider-image-first flow: only fetch Sonarr `remoteUrl` values over HTTPS from `artworks.thetvdb.com` under `/banners/`; if that fails, try Sonarr's same-origin relative poster URL with the API key. Redirects remain disabled and image size/type checks are shared with Radarr.
- Added TV cache counts/progress to the TV library view and regression tests for scheduling, cache-first sequential processing, encrypted storage, URL validation, and UI status.
- Live UI verification found that the coordinator imported `profile-color.mjs` without serving that asset. Added its static route and a regression check so the coordinator module can load and show the TV view.
- Rebuilt only the `metadata` and `roarlay` containers under their separate local deployment locks. Preserved the existing Tailscale Serve root and `/direct-api` handlers.

## Verification

- Docker Rust tests passed: 8 Sonarr-focused tests, including age-encrypted TV storage and strict TVDB poster URL validation. Coordinator and Sonarr scrape tests passed: 17/17; JavaScript syntax check and `git diff --check` passed.
- The first deployed run stored 226 text records but no posters. The TV progress status exposed the problem; adding the safe TVDB remote URL fetch allowed the restarted worker to cache all 226 eligible posters. The Sonarr library has 228 entries; two have no eligible poster.
- The deployed TV page at `https://rawrarr.risk-mermaid.ts.net/#tv` showed 228 live Sonarr series and “Sonarr artwork cache complete”. Shared storage contained 226 encrypted TV text files and 226 encrypted TV artwork files. These are encrypted-file counts, not a claim that every series has poster metadata.
- Metadata and roarlay containers report healthy. The metadata app health endpoint returned HTTP 200; public roarlay `/health` returned its responding page. `tailscale serve status --json` confirmed `/` proxies to `http://127.0.0.1:8080` and `/direct-api` to `http://127.0.0.1:8082`.
- The existing broad Playwright settings/age-ID suite failed its four prior settings/remembered-ID checks (timeouts and a legacy-route assertion); those checks do not exercise the backfill. Focused Sonarr/coordinator tests pass. Host Rust tooling remains unsuitable due the known missing Z3 dylib; release compilation was verified in Docker.

## Recovery

To pause future Sonarr passes, stop spawning the Sonarr worker. Revert the Sonarr-specific storage helpers, progress route and UI, and profile-color static asset route if rolling back the deployed change. Existing Radarr/movie metadata and encrypted Sonarr connection settings are independent. Do not delete the generated shared encrypted TV files during a code rollback.
