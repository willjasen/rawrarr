# Count saved artwork across the library

## Objective

Correct the Movies progress bar after the user observed 900+ encrypted poster files in shared metadata while the bar showed only 109.

## Findings and changes

- The old bar showed posters processed during the current pass, while the shared folder contains posters saved across all passes.
- The backfill worker still rechecked a legacy numeric filename after artwork names had migrated to HMACs. Make it use the vault's HMAC-aware cache lookup, so already saved posters are skipped.
- Track the cumulative shared-cache count against movies with poster URLs. Label the pass counter separately as items checked this pass.
- Serialize the worker and on-demand artwork requests through one lock so Radarr still receives one artwork scrape request at a time.

## Verification

- Focused source regression test passed: `node --test scripts/tests/test_rawrarr_artwork_backfill.mjs`.
- Docker release build completed and only `metadata` was recreated with `--no-deps` under Compose project `rawrarr-local`.
- `http://127.0.0.1:7297/health` returned HTTP 200 and Docker reported the metadata service healthy.
- The live Movies UI displayed `983 / 1,941 posters saved · 11 checked this pass`; the shared folder contained 979 encrypted artwork files during that observation. The values can differ briefly while a poster is being saved or the worker is processing.
- `git diff --check` passed for the changed source and focused regression test.

## Final state

The background artwork job remains enabled. The progress bar now represents cumulative cached posters over the eligible library, with current-pass checks shown separately. The local service is running the updated build.
