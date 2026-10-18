# Movie metadata matching

## Objective and initial state

Extend the local read-only media scan toward a Radarr-style movie library by adding catalog search and user-reviewed matches. The scanner currently lists files but does not identify movies or persist library state. Related context: [local library scan](2026-10-17-local-library-scan.md) and [first Radarr integration](2026-10-17-radarr-first-integration.md).

## Actions

- Added a normalized movie metadata search contract and a Radarr adapter that reuses the configured Radarr API key to query `/api/v3/movie/lookup`.
- Added an on-demand “Find metadata” action for movie rows. It extracts a title/year guess from the filename, displays up to five Radarr candidates, and clearly leaves candidate confirmation and persistence to a later step.
- Added metadata-provider status so lookup controls stay disabled until Radarr is configured. Added TLS support for HTTPS Radarr endpoints.
- Added TMDb attribution and an approved TMDb logo in the UI because Radarr returns TMDb metadata.

## Verification

- `cargo fmt`, locked release build, Compose config, and `git diff --check` passed.
- Local Radarr HTTP mock verified the authenticated lookup route, encoded title/year query, and candidate normalization.
- API smoke checks with no Radarr configuration returned `configured:false` and `503` from movie search as expected.
- Refreshed `http://127.0.0.1:8080/`; the dashboard shows Radarr-dependent metadata lookup guidance. No configured library or live API key was available for a real catalog search.

## Final state and remaining work

Metadata candidates can now be searched after configuring the existing Radarr integration. A library folder and Radarr API key are still required for end-to-end live use. The current scan and suggestions are in-memory only; next steps are persistent library records, confirming a match, fetching full movie details/posters, and coordinating import/download actions. No files or Radarr records were modified.
