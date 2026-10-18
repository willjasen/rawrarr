# Increase Radarr movie table text size

## Objective and initial state

Make the small text in the Movies library table easier to read. Table body cells used a 10px font size.

## Actions

- Increased the shared table cell font size to 16px, affecting movie titles, image load times, artwork sources, and the remaining row values while preserving the existing column headings and dashboard sizing.

## Verification

- `node --test scripts/tests/test_rawrarr_artwork_source.mjs` passed (2 tests).
- `git diff --check` passed for the UI, runbook, and index.
- Rebuilt only the web service under the existing `rawrarr-local` Compose project and confirmed `rawrarr-local-rawrarr-1` is healthy on `127.0.0.1:7297`; `/health` returns HTTP 200.
- The authenticated Movies page was not opened in a browser during this check.

## Final state and remaining work

- Table body cells use a 16px font in the deployed local service. Headers retain their existing styling.
