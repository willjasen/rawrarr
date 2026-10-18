# Keep Movies summary cards on one row

## Objective

Place the four Movies summary cards, including Shared metadata, together on one row.

## Changes

- Added a Movies-specific four-column grid so the shared three-column summary style elsewhere remains unchanged.
- Preserved four columns at the existing tablet breakpoint and allowed the cards to flow to the regular two-column layout on narrow mobile screens.
- Added a focused assertion for the Movies summary grid to the artwork backfill UI regression test.

## Verification

- `node --test scripts/tests/test_rawrarr_artwork_backfill.mjs` passed.
- `git diff --check` passed for the UI and test changes.
- Rebuilt and recreated only `metadata` in Compose project `rawrarr-local` with `--no-deps`; the service is healthy and `/health` returns HTTP 200.
- Reloaded the authenticated Movies page and visually confirmed all four cards occupy one row at the desktop viewport.

## Final state

The local Movies page shows all four summary cards in one row on desktop and tablet layouts.
