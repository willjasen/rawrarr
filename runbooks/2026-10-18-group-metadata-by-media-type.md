# Group metadata by media type

## Objective and initial state

Put media-specific metadata under media-type directories, with separate `text` and `artwork` subdirectories. The existing layout put those kinds first (`metadata/text/movies` and `metadata/artwork/movies`); the shared host folder contains eight movie text ciphertexts and eight poster ciphertexts. Generic library sidecars used `metadata/library`.

## Actions

- Changed movie metadata and artwork read/write paths to `metadata/movies/text/` and `metadata/movies/artwork/`.
- Changed generic encrypted inventory sidecars to `metadata/library/text/`.
- Updated tests, the metadata domain documentation, and the earlier test-scrape runbook path.
- Planned reserved sibling media classes (`tv`, `music`, `books`) follow the same `<type>/{text,artwork}/` structure.
- Stopped the app briefly and moved eight movie metadata ciphertexts and eight artwork ciphertexts to their new paths without decrypting them. Removed the now-empty kind-first directories.
- Created empty `tv`, `music`, and `books` trees with `text` and `artwork` subdirectories; generic scan sidecars now use `library/text/`.

## Verification

- The focused artwork-source Node tests passed (2 tests); Python configuration test syntax compiled; isolated suite passed (9 Node tests, 1 browser test, 11 Rust tests). Python integration tests were skipped because the runner does not provide disposable PostgreSQL fixtures.
- Verified all 16 migrated files still start with the age ciphertext header and no longer exist at the old paths.
- Verified the authenticated movie API reports `shared` for The Dead Girl, Fire in the Sky, and 10 Cloverfield Lane after migration.
- Compose rebuilt the local app on port 7297; `/health` returns `ok` and Docker reports healthy.

## Final state and remaining work

The shared metadata tree is grouped by media type, then by text or artwork. Existing movie records and posters were moved without decryption; no media files were modified.
