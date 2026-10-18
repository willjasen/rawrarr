# Show Radarr artwork load time

## Objective and initial state

Add a browser-observed duration beside each Radarr poster so the page can be used to inspect the time spent fetching shared age-encrypted artwork. The table already showed artwork source; poster elements are created in the browser from the movies API response.

## Actions

- Added a Load time column immediately before Artwork source.
- Measure elapsed browser time from assigning the poster URL through the image `load` event using `performance.now()`. Display milliseconds below one second and seconds otherwise; show `Failed` on image errors and an em dash when no poster URL exists.
- Added focused assertions for column order and load/error timing behavior in `scripts/tests/test_rawrarr_artwork_source.mjs`.

## Verification

- `node --test scripts/tests/test_rawrarr_artwork_source.mjs` passed (2 tests).
- `git diff --check` passed for the UI, focused test, and runbook index.
- Confirmed port 7297 was occupied by the healthy `rawrarr-local` app before rebuilding. Rebuilt only its web service with Compose project `rawrarr-local`; existing PostgreSQL and named data volumes remained in use.
- Confirmed `rawrarr-local-rawrarr-1` is healthy on `127.0.0.1:7297` and `/health` returns HTTP 200.
- The root route is serving the setup/auth page before age-vault unlock, so the authenticated Radarr page and a live image timing value were not checked in the browser. The focused source test checks the column order and browser load/error event behavior.

## Recovery

Rebuild the same `rawrarr-local` Compose app service from the prior source while preserving the existing project name and volumes. Do not remove volumes.

## Final state and remaining work

- The deployed page measures from URL assignment to the image `load` event. This includes request, server-side age decryption, transfer, and image decoding availability reflected by the load event; it cannot isolate decryption time alone.
- After unlocking the existing browser session and opening Movies, load durations appear immediately before Artwork source.
