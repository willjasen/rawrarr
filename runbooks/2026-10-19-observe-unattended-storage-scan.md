# Observe the Mac mini unattended storage scan

## Objective

Verify that the Mac mini storage worker starts a scan without an owner request. Stop the pass after the owner's requested threshold and schedule a fuller scan daily at 4 AM.

## Initial state

- The `rawrarr-mini-storage` container had started at 2026-10-18 18:44:08 EDT, was healthy, and had no container restarts. `RAWARR_STORAGE_SCAN_INTERVAL_SECONDS` was unset, so the application used its 86,400-second default.
- The previous documented Mini scan was an owner-requested, three-file bounded sample. Its three encrypted sidecars had synchronized successfully.

## Observation

- The first scheduled run was due after one interval, at approximately 2026-10-19 18:44 EDT.
- The application logged that automatic scanning was enabled with an 86,400-second interval. No scan-request POST appeared in storage logs after the container start.
- Between 18:44 and 22:46 EDT on 2026-10-19, 403 new `.json.aged` files appeared under `/shared/metadata/library/text/`. Container CPU and block I/O were active and increasing. The owner-authenticated status endpoint returned HTTP 401 from an unauthenticated SSH request; the sidecar count does not distinguish movies from other media. The owner directed that 403 total records be treated as exceeding the 30-movie threshold.

## Stop and schedule

- At the owner's direction, restarted only `rawrarr-mini-storage` at 2026-10-19 22:46:45 EDT. This interrupted the in-progress pass without changing its image, mounts, `/data`, `/shared`, or `/media`.
- After restart, the service returned to `running/healthy`, `/health` returned HTTP 200, and the default automatic scan timer was enabled again. Restarting reset that timer; the next built-in scheduled run is 24 hours after this restart.
- Scheduled the default `storage-scan-once` run daily at 04:00 EDT, starting 2026-10-20, through the Mac mini user's LaunchAgent `com.rawrarr.storage-scan-daily-0400`. It invokes `/usr/local/bin/rust-arr storage-scan-once` inside the existing storage container and writes its output to local-only Application Support logs. The previous one-time job was replaced; the recurring job remains loaded after each run. As a user LaunchAgent, it runs in the Mac mini account's logged-in GUI session. The Mac's `at` service was unavailable, so no `at` job was used.
- Each daily run uses the normal bounded scanner and may report truncation at the 10,000-media-item cap. It is not evidence of a complete cross-node catalog.

## Verification and pending follow-up

- Confirmed the recurring LaunchAgent plist parses, is loaded in `gui/501`, is waiting with `Hour=4` and `Minute=0`, and had not started at 22:54 EDT. Its schedule runs daily.
- Confirmed storage remained healthy after the restart and removed the task's `.container-rebuild-storage.lock` marker.
- The earlier scheduled run was observed starting and writing encrypted sidecars, but was intentionally interrupted. After the first 4 AM run, inspect the local scan log and the new shared scan-completion event, then update this runbook and the roadmap with its counts, errors, and truncation status. Do not claim completion or full-library coverage until that result is verified.
