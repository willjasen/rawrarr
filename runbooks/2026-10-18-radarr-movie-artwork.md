# Show and share Radarr movie artwork

## Objective and initial state

Display Radarr posters in Rawrarr and begin storing Radarr-derived movie metadata in the shared metadata domain. The Movies view previously showed a text table. Shared age-encrypted artwork existed under `/metadata/movies/<tmdb-id>.aged`; this task established separate `artwork/` and `text/` trees. The local service was healthy on `127.0.0.1:7297`.

## Actions

- Added lazy poster thumbnails to the Movies table and an authenticated poster route keyed by TMDb ID.
- The route checks shared encrypted artwork first. On a miss it fetches the corresponding Radarr movie and poster, writes a versioned Radarr text record to `text/movies/<tmdb-id>.json.aged` and poster bytes to `artwork/movies/<tmdb-id>.aged` using the shared age recipient, and serves the response. If shared artwork is missing, it falls back to Radarr.
- Separated shared metadata by kind: human-readable structured text records live below `metadata/text/`; binary media assets live below `metadata/artwork/`. Both remain encrypted and share the same synchronized root.
- Restricted remote poster sources to HTTPS `image.tmdb.org`; Radarr-local image paths are resolved against the configured Radarr origin, API-key requests do not follow redirects, and image responses are capped at 8 MiB.
- Documented the version 1 metadata record fields and on-demand scraping behavior in [data domains](../docs/data-domains.md), [design](../docs/DESIGN.md), and [technical status](../docs/technical-status.md).
- Updated the Rust encryption round-trip test to assert distinct text and artwork paths and ciphertext.

## Verification

- Focused build and tests: pending.

## Final state and remaining work

Implementation is pending build, unit-test, and live UI verification. Live Radarr poster retrieval has not yet been confirmed.
