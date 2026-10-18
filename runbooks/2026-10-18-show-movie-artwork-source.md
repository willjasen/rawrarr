# Show movie artwork source

## Objective and initial state

Show whether each poster in the Radarr Movies table is loaded from Rawrarr's shared encrypted artwork cache or will be loaded from Radarr. The table displayed posters but gave no source status. Shared artwork is stored under `metadata/artwork/movies/<tmdb-id>.aged`.

## Actions

- Added `artworkSource` to the Radarr movies API response, based on whether the shared encrypted artwork file exists for that movie.
- Added an Artwork source column showing `Shared folder`, `Radarr`, or `Unavailable` when the poster request fails.
- Added a focused Node test for API source classification and UI states.

## Verification

- The focused Node tests passed (2 tests); the full isolated suite passed (9 Node tests, 1 browser test, 11 Rust tests). Python integration tests were skipped by the isolated runner because they require disposable PostgreSQL fixtures.
- Rebuilt the local app after confirming port 7297 belonged to the healthy `rawrarr-local` container. Docker reports the updated service healthy and `/health` returns `ok`.
- The authenticated movie API response reports `shared` for The Dead Girl, Fire in the Sky, and 10 Cloverfield Lane, and `radarr` for uncached movies. This verifies the source field against the three previously scraped records.
- The browser page required age-vault unlock after the container restart; no age identity was entered. The authenticated movie API was queried after deployment and returned the expected shared source for all three cached movies. The table markup and status states are covered by the Node test.

## Final state and remaining work

The app is deployed on `127.0.0.1:7297`. After unlocking the age vault in the browser session, the Movies page will show the new Artwork source column.
