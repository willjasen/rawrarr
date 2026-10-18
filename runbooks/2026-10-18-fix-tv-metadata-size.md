# Fix TV metadata size

## Objective and initial state

The Movies and TV cards displayed the same media subtotal because the coordinator UI assigned both cards `moviesBytes`. The metadata tree has separate `movies/` and `tv/` subtrees.

## Changes

- Added `tvBytes` to `GET /api/metadata/storage`, measured from `metadata/tv`.
- Wired the TV card to `tvBytes`; the overall metadata total remains shared.
- Added a focused regression test for separate API and card wiring.

## Verification

- `node --test scripts/tests/test_rawrarr_metadata_sizes.mjs` passed (1 test), including the TV subtitle removal.
- `git diff --check` passed.
- Rebuilt the metadata and roarlay services in the existing `rawrarr-local` Compose project. Both report healthy; local and tailnet health endpoints return HTTP 200.
- Confirmed Tailscale Serve keeps HTTPS `/` routed to `http://127.0.0.1:8080` and preserves `/direct-api` at `http://127.0.0.1:8082`.
- The metadata storage API is authenticated; an unauthenticated direct probe returned HTTP 401. Browser-visible size values were not read back.

## Final state

The storage API reports distinct Movies and TV subtree sizes. The TV card reads the TV-specific value and no longer shows the “Movies and TV metadata” subtitle. Both changes are deployed. The worktree remains uncommitted because it contains extensive pre-existing changes.
