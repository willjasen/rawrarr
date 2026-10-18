# Move route latency to Devices

## Objective

Show measured direct Tailscale latency on device cards and remove the route-status messages from Movies and TV.

## Initial state

Movies and TV each displayed a route-status banner, including direct Tailscale latency. The Devices page already probed each node and had its measured latency available. The worktree contains extensive pre-existing changes; preserve them.

## Changes

- Added the direct route latency beside each Direct label on Devices, with an accessible label that includes the route and latency.
- Removed the Movies and TV route-status banners and their unused styling and message updates. Route probing, direct-session selection, and relay fallback remain in place.
- Rebuilt and updated only roarlay in the existing `rawrarr-local` Compose project using the active base, coordinator, and MacBook Tailscale Compose files.
- Fixed the storage scan's `spawn_blocking` result conversion so the build can return a `Send`-compatible error. The duplicate `storage_scan` declaration observed on the first build was already absent when the deployment lock became available.

## Verification

- `node --check rust-arr/src/coordinator-workspace.mjs` passed.
- `git diff --check` passed for tracked changes.
- The coordinator workspace JavaScript served by roarlay contains the Devices latency label.
- Roarlay reports healthy. `http://127.0.0.1:7297/health` and `https://rawrarr.risk-mermaid.ts.net/health` respond and identify Roarlay.
- Tailscale Serve readback confirms HTTPS `/` proxies to `http://127.0.0.1:8080`; the existing `/direct-api` proxy remains `http://127.0.0.1:8082`.
- No automated tests were run.

## Recovery

Revert the Devices latency label and removal of route-status UI in `rust-arr/src/coordinator-workspace.mjs` and `rust-arr/src/coordinator_workspace.html`. Revert the error conversion in `rust-arr/src/storage_node.rs` if no longer needed. No service data or configuration was changed.
