# Browser-triggered and daily storage scans

## Objective

Let the signed-in owner request missing encrypted sidecar metadata from a storage node in the Devices view, while storage nodes automatically check for missing metadata once per day.

## Environment and initial state

- Repository: Rawrarr Rust service and coordinator workspace.
- Existing working tree contains changes from other tasks. This task edits only the scan service, its coordinator route/UI, focused tests, documentation, and this runbook; no commit was created because the touched files are shared with other uncommitted work.
- Storage media is mounted node-local and read-only; synchronized output remains age-encrypted under `/shared/metadata/library/text/`. Scan cache is node-local under `/data` and contains hashed relative paths with size/mtime fingerprints.
- The prior 1–2 GB live Mini scan was not run: SSH to the Mini was refused and the currently deployed service could not be confirmed to contain the Rust scan implementation.

## Implementation

- Added owner-session-protected `POST /api/library/scan/missing` and `GET /api/library/scan/status` endpoints on storage nodes. Coordinator relay accepts only those exact paths for storage-role nodes, validates the request body, and retains the normal owner authorization/session checks.
- Added Devices actions for a missing-metadata scan and a sample capped at three movies from 1–2 GB, with asynchronous progress and completion counts.
- Storage nodes begin the scheduled scan one interval after startup, then repeat every 24 hours by default. `RAWARR_STORAGE_SCAN_INTERVAL_SECONDS` can set another interval from 60 seconds to seven days or disable scheduled scans with `0`.
- Scans reuse an opaque `/data` cache and verify the encrypted target sidecar exists before skipping unchanged media. Changed media or missing sidecars are hashed/probed and re-encrypted under `/shared/metadata/library/text/`.
- Updated deployment and architecture documentation.

## Verification

- In `rawrarr-testing-base:local`, `cargo test --locked --offline storage_node::tests` passed all six tests; the relay authorization test passed (`only_owner_authenticated_storage_scan_requests_can_use_the_relay`).
- `node --test scripts/tests/test_rawrarr_storage_scan_ui.mjs` passed. `python3 -m py_compile scripts/tests/test_rawrarr_configuration.py` and `git diff --check` passed.
- `node --test scripts/tests/test_rawrarr_coordinator_workspace.mjs` had 10 passing and 4 failing assertions in existing Sonarr label, coordinator landing, and route-status expectations. Those assertions do not cover the storage scan additions.
- Live Mini deployment, browser request against the deployed Mini, and encrypted shared-folder replication were not verified because remote shell access was unavailable. No live media was scanned or written.

## Recovery and remaining work

- Disable scheduled scans with `RAWARR_STORAGE_SCAN_INTERVAL_SECONDS=0` if needed. Remove only node-local `/data/storage-sidecar-cache.json` to force a future rescan; it is a performance cache, not authoritative metadata.
- Deploy the updated storage image to the Mac mini, then run the bounded 1–2 GB sample from Devices and inspect/decrypt the resulting records before relying on the report.
- The user-requested live sample scan and shared-folder result report remain outstanding pending Mini deployment/access.
