# Opaque movie metadata filenames

## Objective and initial state

Replace numbered movie metadata and artwork filenames with HMAC-derived names while retaining decryptable ciphertext and restart the sequential artwork scraper on the metadata node. The existing encrypted config already contained a 256-bit OS-random `metadata_filename_key`; the shared tree had 1,941 numbered text records and 818 artwork files. The background scraper had been paused by removing its startup launch.

## Actions

- Added HMAC-SHA-256 naming with domain separation (`rawrarr-metadata-filename-v1`, content kind, `tmdb`, numeric ID). Text uses `<64 lowercase hex>.json.aged`; artwork uses `<64 lowercase hex>.aged`.
- Updated encrypted movie metadata/artwork readers and writers to prefer the HMAC path and fall back to the legacy numeric path.
- Added a metadata-node migration that copies each numeric ciphertext, byte-verifies the copy, renames it to the opaque path, and removes the legacy name only after verification. Conflicting opaque content leaves the numbered file intact.
- Re-enabled the sequential metadata-node artwork worker. It first migrates existing files, then fetches uncached Radarr posters one at a time, showing progress in Movies.
- Added focused Rust coverage for domain separation, filename shape, ciphertext-preserving migration, and decryption from the migrated paths.

## Verification

- Docker release build completed and only `rawrarr-local`'s metadata service was recreated. The service is healthy and `http://127.0.0.1:7297/health` returns HTTP 200.
- The shared tree now contains 1,941 HMAC-named movie text files and 882 HMAC-named poster files, with zero remaining numeric movie filenames. This preserves the existing ciphertext and removes TMDb IDs from movie text/artwork names; the artwork total continued growing during verification.
- The authenticated Movies page visibly reports the active job at `43 / 1,117 posters scraped`; artwork count continued increasing while verification ran.
- The focused HMAC Rust test passed inside the isolated suite; all 34 Rust unit tests passed. Playwright passed. Five unrelated existing Node tests failed (four artwork timing tests due to `window` missing in the test VM and one titleSlug assertion); the new backfill/source Node checks passed.
- Host `cargo fmt` remains unavailable because the installed Rust toolchain aborts on its missing local Z3 dylib; the container release build and isolated Rust suite compile successfully.

## Recovery and remaining work

Do not rotate or discard the encrypted filename key. If an opaque path conflicts with different bytes, the migration leaves the legacy numeric file untouched for manual inspection. The backfill is enabled on the local metadata node and continues sequentially, with a six-hour retry interval after each completed pass. Confirm the filename key and an HMAC-capable Rawrarr build are synchronized to other metadata/storage nodes before they consume opaque-named files.
