# Sonarr integration and shared library layout

## Objective and initial state
Add read-only Sonarr series integration and make library column layout shared with Radarr. Existing worktree has extensive unrelated edits, preserved. Related: [Radarr connection save](2026-10-18-radarr-connection-save.md), [responsive movies](2026-10-18-responsive-movies-refresh.md).

## Design
Use one column schema and renderer for both libraries, retaining provider-specific identifiers and availability semantics. Sonarr credentials stay in subscriber age-encrypted configuration. Sonarr v3 API (Sonarr 3/4) is the initial target: https://sonarr.tv/docs/api/ . Episode mutation and shared encrypted television artwork caching remain planned.

## Actions and verification
Implementation in progress.

## Implemented
- Shared `libraryColumns` and `renderArrLibrary` for Radarr and Sonarr. Provider descriptors supply identifier, title, availability, and detail-link semantics; column order and formatting have one owner.
- Sonarr connection form, authenticated settings/status/series routes, HTTPS transport reuse (`ArrClient`, with a compatibility Radarr alias), service-name validation, read-only series/quality-profile reads, and normalized episode statistics.
- Optional backward-compatible Sonarr config field and serialized encrypted save/clear that preserves Radarr and the filename key. Sonarr is not added to the optional Radarr plaintext mirror.
- Sonarr artwork, metadata lookup, and snapshots remain planned; its artwork column explicitly shows unavailable.

## Verification
- Release Docker build passed.
- Three Sonarr Rust tests and one real age-encryption save/clear preservation test passed.
- Ten focused Node/Chromium checks passed across movie links, responsiveness/snapshots, artwork-source assertions, and shared library behavior. The shared-layout test reorders the column schema and confirms both tables change. The old static-header assertion was updated for the schema.
- `git diff --check` passed. No live Sonarr endpoint or credentials were supplied; live connectivity remains unverified.

## Deployment limitation
The existing app was restarting before this task (`Set RAWARR_ROLE explicitly`). The required Compose rebuild succeeded, but `up` failed because concurrent node-role changes require a missing coordinator approved-nodes file. Another deployment subsequently restored a healthy app: `/health` returned 200. Its Compose metadata points to a temporary movie-deployment configuration, and its binary does not contain the shared renderer, so this task's UI is NOT confirmed deployed. Do not overwrite that concurrent recovery or provision credentials as a side effect of Sonarr work. Finish node-role provisioning before deploying this source with the normal `rawrarr-local` Compose command.

## Recovery and remaining work
Reverse only the Sonarr/shared-layout changes if needed, preserving all pre-existing work and runtime volumes. Integration edits in mixed-ownership source files remain uncommitted; new task-owned modules/tests/runbook are committed separately. No remote is configured. Complete persistent deployment once the concurrent node-role migration is ready, then connect Sonarr from Settings and verify a live series list.
