# Show storage scan progress

## Objective

Show an accessible indeterminate progress bar with “Storage scan is running…” on storage-node cards while a scan is active.

## Initial state

- The Devices page polled scan status and displayed a text-only running message.
- The storage API reports whether a scan is active and only returns file counts on completion, so it does not provide a truthful numeric percentage.

## Actions

- Added a moving progress indicator beneath the storage scan status. It is hidden when idle, has progressbar semantics and an accessible running label, and becomes a solid indicator for reduced-motion preferences.
- Wired the indicator visibility to the existing per-node scan-active state, including restored device cards and the already-running response path.

## Verification

- `node --check rust-arr/src/coordinator-workspace.mjs` and `git diff --check` passed.
- Rebuilt only `roarlay` in the existing `rawrarr-local` Compose project. The container is healthy, local and public `/health` identify Roarlay, and Tailscale Serve still maps `/` to `127.0.0.1:8080` and `/direct-api` to `127.0.0.1:8082`.
- Reloaded the signed-in Devices page, started the bounded three-movie scan, and visually confirmed the Mac mini card displays “Storage scan is running…” with an animated progress bar and both scan buttons disabled.
- The scan completed with 3 files checked, 0 written, 3 already complete, 0 errors; the progress bar is hidden again on completion.

## Final state

- The accessible indeterminate progress bar is deployed on Roarlay. The API does not report incremental counts, so the UI does not imply a numeric percentage. Reduced-motion preferences receive a static indicator. The container deployment lock was released after health and route verification.
