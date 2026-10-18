# Share Arr summary card rendering

## Objective and initial state

The Movies and TV pages duplicated four metric cards in HTML and updated values through page-specific IDs. Both metadata cards also described their subtotal as “Movies and TV metadata.”

## Changes

- Added one renderer and update/reset helpers for Arr summary card rows.
- Defined Radarr and Sonarr labels, detail text, value keys, and metadata-total secondary values in service configurations.
- Replaced duplicated HTML card markup with empty row containers rendered from those definitions.
- Removed the misleading shared-media subtitle and use service-specific metadata descriptions.
- Added focused regression coverage for the shared renderer and service-specific definitions.

## Verification

- `node --test scripts/tests/test_rawrarr_arr_summary.mjs scripts/tests/test_rawrarr_metadata_sizes.mjs` passed (2 tests).
- `git diff --check` passed.
- The coordinator workspace regression file still fails an earlier existing assertion about `tv-route-status` markup, before its summary assertions run.
- Rebuilt roarlay in the existing `rawrarr-local` project. It reports healthy; local and tailnet health endpoints return HTTP 200. Tailscale Serve retains the coordinator and direct API routes.

## Final state

Radarr and Sonarr now render their four summary cards through the shared config-driven renderer. Future Arr summaries can add a service definition and reuse the same card renderer and value/reset helpers. The updated cards are deployed to roarlay. Work remains uncommitted because the repository contains extensive pre-existing changes.
