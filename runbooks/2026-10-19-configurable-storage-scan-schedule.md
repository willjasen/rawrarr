# Configure scheduled storage scans from Devices

## Objective

Show storage schedule state in the coordinator Devices view and let the owner select a daily or weekly local-time schedule.

## Design

- The storage node owns and executes its schedule. Schedule JSON and the last claimed schedule slot stay in node-local `/data`; roarlay relays a narrowly scoped API and never accesses `/media`.
- The schedule specifies daily or weekly frequency, a 24-hour wall-clock time, and an IANA time zone. Initial default is daily at 04:00 in `RAWARR_STORAGE_SCAN_TIMEZONE` (UTC if unset).
- The owner-authenticated Devices card shows the next scheduled run and current scheduled-scan progress. It saves changes to the storage node over the existing direct route or the role-checked relay.
- The scanner remains bounded by its existing traversal caps, runs only one scan at a time, and writes encrypted sidecars to `/shared`.
- A nonexistent local time during a daylight-saving transition skips that occurrence; a repeated local time runs only at its first occurrence. Offline nodes do not replay missed slots. A scan already running at the scheduled minute is not interrupted or overlapped.

## Implementation

- Replaced the elapsed-interval timer with a wall-clock scheduler that persists configuration and its last scheduled start under node-local `/data`.
- Added storage-only schedule GET/PUT routes, strict frequency/time/time-zone validation, daily/weekly calculation, local-time/DST-aware next-run handling, and inclusion in the coordinator's owner-authenticated relay allowlist.
- Added per-storage-device frequency, weekday, and time controls plus next-run and scheduled-progress status in Devices.
- Updated standalone Compose examples, design, deployment, architecture, and user guidance. Storage-only schedule data and local media remain off the coordinator.

## Verification

- `node --check rust-arr/src/coordinator-workspace.mjs`, the three `test_rawrarr_storage_scan_ui.mjs` tests, and `git diff --check` passed. Executable UI fixtures cover restored unsaved drafts, scheduled progress counts, weekly PUT payloads, and successful draft clearing.
- The 8 focused Rust schedule tests and the storage-only coordinator relay authorization test passed with Rust 1.85.1.
- The complete Rust suite compiled and ran 65 tests: 60 passed, including the new schedule tests. Five unrelated existing tests failed (direct-route acceptance, two missing avatar encoder tools, and two quality-profile fixture expectations).
- The host's default Rust compiler cannot start because Homebrew LLVM requires a missing `libz3.5.1.dylib`; the standalone Rust 1.85.1 compiler was used for tests. A repository-wide formatting check reports pre-existing formatting differences across unrelated and modified source/test files.

## Deployment

- Built and deployed the Mac mini `storage` service in its existing `rawrarr-local` project under the storage deployment lock. Its existing data volume, read-only media bind, and shared bind were preserved. The persistent schedule is daily at `04:00`, `America/New_York`; the service is healthy with zero restarts after final recreation.
- The Mini Compose configuration pointed to `https://swarmsail.willjasen.net`, whose enrollment endpoint returned HTTP 502 on restart. Restored the subscriber's transport to its actual Roarlay coordinator at `https://rawrarr.risk-mermaid.ts.net` through the existing Tailscale SOCKS5 listener. The Mini registered successfully and its recent heartbeat is visible in the coordinator.
- Backups of the Mini source and Compose config are under `~/Library/Application Support/rawrarr/deploy/backups/`, with names ending `before-scan-schedule-20261019`. After verifying the replacement scheduler and health, unloaded `com.rawrarr.storage-scan-daily-0400` and archived its plist in the same backup directory. Only the service scheduler now owns the daily run.
- Built Roarlay from the already-deployed AI branch snapshot `73eeaed` with this task's scheduling patch applied in an isolated staging worktree. This preserves the existing AI features without modifying their source worktree. Recreated only `roarlay` with its original Compose file set and `--no-deps --no-build`; metadata, other storage services, PostgreSQL, Tailscale, and Syncthing were not restarted.
- Roarlay is healthy; local port 7297 and tailnet `/health` both identify Roarlay. Serve readback retains `/` -> `http://127.0.0.1:8080`, the direct API handler, and Syncthing TCP forwarding. The deployed browser module contains the schedule controls. Mini public health succeeds, and its Serve handlers are unchanged. Released both deployment markers after verification.
- The browser canvas is signed out, so authenticated live schedule editing and visual verification were not performed. No full scan or near-term scheduled media scan was triggered for verification. The next intended Mini run remains 04:00 New York time; completion and full-library coverage remain unverified.
