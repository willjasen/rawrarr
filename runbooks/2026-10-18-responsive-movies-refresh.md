# Responsive Arr library loading and refresh

## Objective and findings

Improve the Movies page, which initially displayed “Loading movies from Radarr…” during page reloads and refreshes. Testing the existing Chrome session showed 1,941 movies and exposed two separate waits:

- After receiving the movie response, the browser awaited a Radarr URL reachability probe before drawing rows. This probe is only needed to enable links to Radarr.
- Fetching the current list could take several seconds. A failed in-page update also needed to preserve the usable prior list.

The first fixes removed the link-probe wait, preserved rows during refresh, and cached decrypted quality-profile metadata in the unlocked process. Chrome still waited for the live list after a full reload, so the final change added a last-successful-list snapshot held in server memory.

## Behavior and data lifetime

- Draw list rows and posters as soon as a live API response is available; resolve optional Radarr/Sonarr links separately.
- Keep the current rows visible during manual refresh and report an update error without discarding them.
- Keep the latest successfully loaded Radarr list in memory on its `RadarrClient`. The authenticated snapshot endpoint returns its rows and update time with `Cache-Control: no-store` while the live list request runs.
- On a page reload, request the snapshot and fresh Radarr data concurrently. Show the timestamped snapshot immediately, then replace it with live data. Ignore late snapshot responses once live results have arrived.
- Replacing or clearing the Radarr client drops its snapshot. Process restarts also clear it, so the first successful load after restart or reconnection still waits for Radarr. The snapshot is not written to browser storage, `/data`, or `/shared`.
- Decrypt metadata on cache misses with at most eight workers. Memoize metadata only in the unlocked vault process, keyed by the ciphertext SHA-256. Check current ciphertext each read so synchronized file changes invalidate the memoized value; bound the cache at 4,096 entries.

## Verification

- Eight focused Chromium tests passed for pending link probes, movie rendering and posters, retained rows after errors, reload snapshot display, late-response ordering, links, and workspace navigation.
- The Rust snapshot-lifetime test passed. The preceding quality-profile/cache tests passed: two cache tests and five quality-profile tests.
- The local release image built successfully from an isolated source context because a concurrent node-role and Compose migration was in progress. That migration changed authentication modules and Compose configuration during the task. The isolated deployment retained the existing app port, data volume, shared-data bind, and password-file mount; it did not recreate volumes or remove orphaned services.
- In Chrome, all 1,941 rows appeared in approximately 1.56 seconds from a full reload, labeled with the snapshot time and “Updating…”, then changed to “live data from Radarr.” Browser automation overhead is included. The service health endpoint returned HTTP 200 and its container reported healthy.
- Screenshots: `/tmp/rawrarr-movies-chrome-verified.jpg` (live list) and `/tmp/rawrarr-movies-snapshot-verified.jpg` (snapshot during refresh).

## Recovery and future deployment notes

The source changes are committed in `e9da21c` (quality-profile cache) and `c3ebf96` (movie snapshot). The page renderer also includes related uncommitted UI work in the shared worktree; preserve it when resolving or deploying the concurrent branch changes.

The task's isolated Compose manifest was generated from the running container's non-secret settings and mounts at a temporary path recorded in `/tmp/rawrarr-movie-deploy-path`. It was a recovery aid for this deployment incident, not a replacement project configuration. The deployment image was later replaced during concurrent work, so do not rely on its old image ID. Before a future rebuild, inspect the now-current node-role Compose files and their configured health dependencies, ensure the app is assigned its intended role, then build and health-check the existing `rawrarr-local` service on `127.0.0.1:7297` without removing its volumes.
