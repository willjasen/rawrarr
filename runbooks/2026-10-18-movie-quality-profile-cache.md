# Resolve movie quality profiles from shared metadata

## Objective and initial state

Movies renders an empty quality profile because the adapter expects an embedded profile name, while Radarr supplies a profile ID. Existing encrypted movie metadata lives under `metadata/movies/text/<tmdb-id>.json.aged`.

Related: [Metadata layout](2026-10-18-group-metadata-by-media-type.md). Existing and concurrent checkout changes must remain separate.

## Actions

Read shared metadata first; resolve missing or changed profile IDs using one Radarr profile-list request, merge into encrypted metadata, and preserve other fields. Test cache precedence, misses, persistence, stale IDs, and failures. Rebuild the persistent local app and check health.

## Verification

- All 16 Rust tests passed in the isolated test image, including four new quality-profile regression tests and the existing real age ciphertext round-trip test.
- Rebuilt only `rawrarr-local` app using its protected password-file path and `--no-deps`; preserved PostgreSQL, data, and browser windows. Docker reports healthy; `/health` returns HTTP 200.
- Clicked Refresh movies in the existing authenticated browser: all 1,941 rows now display a quality-profile name, with zero missing values.
- Verified all 1,941 shared movie metadata files have age ciphertext headers and were updated during the first fetch. A subsequent successful refresh displayed all 1,941 profiles again with zero metadata rewrites, confirming cache reuse. One intervening movie-list request failed upstream; a retry succeeded.
- Local rustfmt could not start because its LLVM dependency references a missing Z3 library; the isolated image does not include rustfmt. Compilation and tests succeeded.

## Recovery and limits

Revert only this task's source changes and rebuild the app. The added metadata fields are backward-compatible and can remain. Existing metadata cannot be read while the age vault is locked; the API reports an explicit unlock/read error instead of overwriting it. Unknown upstream IDs remain blank and are not cached. The movie listing itself still comes from Radarr.

## Git

Task-only commit on `main`; no upstream is configured, so no push. Unrelated existing and concurrent changes remain outside this task.
