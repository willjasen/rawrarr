# Automatically backfill missing movie artwork

## Objective and initial state

Check whether Rawrarr had movie posters that were not yet saved in shared metadata, backfill them, and make future backfills automatic. The local metadata service was healthy at `http://127.0.0.1:7297`. Shared metadata contained 517 encrypted movie artwork files and 1,941 encrypted movie text records. Radarr reported 1,941 library entries; the Movies view showed 104 Radarr-sourced posters in its first 500 entries. The metadata node intentionally has no local Radarr credential mirror.

## Actions

- Added a startup background worker on metadata-role nodes. It selects only movies with a poster URL and `artworkSource` set to `radarr`, checks the shared artwork cache again before each fetch, and resumes after restart by skipping cached files. It makes one Radarr artwork request at a time and waits six hours between completed passes.
- The worker processes one movie at a time, using Radarr's existing API client and shared age-encrypted metadata writer. Concurrent trials caused Radarr's five-second client timeout and were discarded.
- Added an authenticated progress endpoint and Movies progress bar showing scraped count, total missing posters, and failures.
- Added a focused source-level regression test and indexed it.

## Verification

- The authenticated Movies view reported 1,941 live Radarr entries. Before this worker, shared metadata had 517 encrypted artwork files and 1,941 encrypted text records. Earlier browser-demand requests brought artwork to 688 before the sequential worker deployment.
- Focused source regression test passed: `node --test scripts/tests/test_rawrarr_artwork_backfill.mjs`.
- Docker release build passed and only the `metadata` service was recreated with `--no-deps`. `http://127.0.0.1:7297/health` returned HTTP 200; Docker reports the metadata container healthy.
- After startup, the encrypted artwork file count reached 772 while text metadata remained 1,941. The worker is continuing in the background. The browser automation connection timed out while trying to inspect the progress bar, so visual rendering and live endpoint values were not independently verified in this session.
- Host `cargo check` remains unavailable because the installed Rust toolchain aborts loading a missing local Z3 dylib; the container release build provides the compile check instead.

## Recovery and remaining work

The worker uses credentials already configured on the metadata node; it does not copy them to storage or coordinator nodes or write plaintext art. The browser-triggered queue was removed. The background worker retries remaining items on its next six-hour pass or after a service restart; the Movies progress bar reports the active pass. Counts are best-effort from the most recent Radarr inventory and the worker's in-memory pass state.
