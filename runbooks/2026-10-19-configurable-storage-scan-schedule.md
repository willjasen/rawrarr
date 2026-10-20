# Configure scheduled storage scans from Devices

## Objective

Show storage schedule state in the coordinator Devices view and let the owner select a daily or weekly local-time schedule.

## Design

- The storage node owns and executes its schedule. Schedule JSON and the last claimed schedule slot stay in node-local `/data`; roarlay relays a narrowly scoped API and never accesses `/media`.
- The schedule specifies daily or weekly frequency, a 24-hour wall-clock time, and an IANA time zone. Initial default is daily at 04:00 in `RAWARR_STORAGE_SCAN_TIMEZONE` (UTC if unset).
- The owner-authenticated Devices card shows the next scheduled run and current scheduled-scan progress. It saves changes to the storage node over the existing direct route or the role-checked relay.
- The scanner remains bounded by its existing traversal caps, runs only one scan at a time, and writes encrypted sidecars to `/shared`.

## Implementation

- Replaced the elapsed-interval timer with a wall-clock scheduler that persists configuration and its last scheduled start under node-local `/data`.
- Added storage-only schedule GET/PUT routes, strict frequency/time/time-zone validation, daily/weekly calculation, local-time/DST-aware next-run handling, and inclusion in the coordinator's owner-authenticated relay allowlist.
- Added per-storage-device frequency, weekday, and time controls plus next-run and scheduled-progress status in Devices.
- Updated standalone Compose examples, design, deployment, architecture, and user guidance. Storage-only schedule data and local media remain off the coordinator.

## Verification

- `node --check rust-arr/src/coordinator-workspace.mjs`, `node --test scripts/tests/test_rawrarr_storage_scan_ui.mjs`, and `git diff --check` passed.
- The 8 focused Rust schedule tests and the storage-only coordinator relay authorization test passed with Rust 1.85.1.
- The complete Rust suite compiled and ran 65 tests: 60 passed, including the new schedule tests. Five unrelated existing tests failed (direct-route acceptance, two missing avatar encoder tools, and two quality-profile fixture expectations).
- The host's default Rust compiler cannot start because Homebrew LLVM requires a missing `libz3.5.1.dylib`; the standalone Rust 1.85.1 compiler was used for tests. A repository-wide formatting check reports pre-existing formatting differences across unrelated and modified source/test files.

Pending deployment: build/recreate only `roarlay` in the existing `rawrarr-local` Compose project and verify its health and tailnet route. Update the Mac mini storage image with the new schedule API/scheduler under its storage deployment lock, configure `RAWARR_STORAGE_SCAN_TIMEZONE=America/New_York`, verify it is healthy and exposes the saved daily 04:00 schedule, then remove the temporary LaunchAgent created for the one-time run. Preserve `/data`, `/shared`, `/media`, Syncthing, and Tailscale state.
