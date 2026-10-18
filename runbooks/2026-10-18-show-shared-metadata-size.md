# Show shared metadata size on Movies

## Objective and initial state

Show the current size of the encrypted shared metadata tree on the Movies page and keep it current while the metadata backfill runs. The page already polled artwork progress every three seconds, but did not show the total storage used by artwork and text metadata.

## Changes

- Added authenticated `GET /api/metadata/storage`, which recursively sums regular-file sizes under the configured metadata directory without reading file contents.
- Added a Movies summary card labeled **Shared metadata**, displayed in binary units, and poll it every three seconds.
- Added endpoint and UI assertions to the existing focused artwork-backfill regression test.

## Verification

- `node --test scripts/tests/test_rawrarr_artwork_backfill.mjs` passed.
- `git diff --check` passed for source and test changes.
- Rebuilt and recreated only the metadata service in Compose project `rawrarr-local` using `--no-deps`; the release build succeeded.
- `http://127.0.0.1:7297/health` returned HTTP 200 and Docker reported the metadata service healthy.
- Container inspection confirmed `/metadata` resolves to `/shared/metadata`, with movie artwork files present.
- The authenticated Movies page displayed **671.3 MB** for shared metadata while artwork progress read **1,191 / 1,941**. This verifies the size is visible alongside live scrape progress.

## Final state

The local Rawrarr metadata service serves the size endpoint. The authenticated Movies page polls it every three seconds. No metadata files are modified by the endpoint.
