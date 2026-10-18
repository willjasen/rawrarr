# Automatic scanning on storage nodes

## Objective and initial state

Add an unattended scanner to the enrolled storage service on the Mac mini. The existing `storage-scan-once` utility already scans one configured root and writes age-encrypted sidecars using the shared public recipient. Enrollment, heartbeat, and the direct Mini route exist in the current working tree. The repository had extensive pre-existing edits; all unrelated changes remain untouched.

## Implementation

- Storage nodes now start a background scan after coordinator registration succeeds. The initial scan runs immediately; later scans wait for the configured interval, so slow runs do not overlap.
- `RAWARR_STORAGE_SCAN_INTERVAL_SECONDS` defaults to six hours, accepts 60 through 604800 seconds, and accepts `0` to disable. The storage Compose file sets the default explicitly.
- Scanning runs in a blocking worker thread and reuses existing traversal caps and public-recipient encryption. It does not need PostgreSQL or an age private identity.
- Completed counts go to existing shared operational logs. Failures and incomplete/truncated results are reported without printing paths or scanning errors, then retried after the interval. The scanner never removes sidecars, so a failed or empty scan cannot turn into mass deletion.
- Updated deployment guidance and the storage-node plan to distinguish the implemented periodic scan from the still-planned catalog ingestion and reconciliation.

## Verification

The focused Rust tests passed (3/3). `git diff --check` passed. The repository-wide `cargo fmt --check` reports extensive formatting differences across pre-existing edited files, so no whole-tree formatting was applied. The host Rust compiler is unusable because its linked LLVM cannot load `libz3.5.1.dylib`; tests were compiled and run in the existing `rawrarr-testing-base:local` container image.

The Mini health route returned HTTP 200. Deployment could not be performed: SSH to `rawrarr-mini.risk-mermaid.ts.net:22` is refused, and the local Docker context list contains only this Mac's OrbStack/Desktop contexts. The Mini's apex deployment lock, image, and shared-folder output therefore could not be inspected or changed. The live Mini still needs a rebuild and an encrypted-sidecar verification before periodic scanning can be claimed as active there.

## Recovery and remaining work

Set `RAWARR_STORAGE_SCAN_INTERVAL_SECONDS=0` and restart the storage service to stop scheduled scans while retaining the node, volume, media, and shared data. Remaining: rebuild the existing `rawrarr-local` `apex` service under its deployment lock and verify a new encrypted sidecar reaches the shared folder. Coordinator catalog ingestion remains separate work.
