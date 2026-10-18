# Fix Overview metadata node selection

## Objective

Ensure Overview uses an available metadata node when an apex node is selected, and accurately distinguish an unselected metadata node from no online metadata node.

## Initial state

- Chrome showed both metadata and apex devices online, with the apex node selected; Overview incorrectly said “No metadata node.”
- The coordinator preserves its selected node in browser session state. Overview data requires a metadata node.
- The checkout contains extensive unrelated changes and untracked work; only task-owned edits should be staged.

## Actions

- Updated coordinator selection handling to fall back to the available metadata node when the saved selection is an apex node.
- Changed the Overview empty state to distinguish an available but unselected metadata node from no online metadata node.
- Added focused regression assertions for both behaviors.

## Deployment and verification

- The first single-file Compose invocation failed because it attempted to publish port 7298 already owned by `rawrarr-macbook-tailscale`. Re-ran with the existing `docker-compose.yml`, `docker-compose.coordinator.yml`, and `docker-compose.tailscale-macbook.yml` configuration so roarlay shares the Tailscale network namespace and publishes no competing port.
- Focused regression checks passed: `node --test --test-name-pattern='overview distinguishes|apex selection falls back' scripts/tests/test_rawrarr_coordinator_workspace.mjs` (2 tests).
- The full coordinator workspace suite has 11 passing and 5 failing assertions against unrelated existing changes (TV route status, auth copy, route labels, and direct-route status text).
- `rawrarr-local-roarlay-1` is healthy. `https://rawrarr.risk-mermaid.ts.net/health` returned HTTP 200 and identified Roarlay.
- Tailscale Serve JSON confirms `/` proxies to `http://127.0.0.1:8080`; the existing `/direct-api` handler remains configured. Chrome live verification showed MacBook Pro selected and Overview displaying `idle`, `100% synced`, 1.2 GB shared metadata, and Radarr statistics.
- The stale zero-byte rebuild marker had no process holding it and was several hours old. It was replaced with this task's atomic marker and removed after deployment verification.

## Recovery

- Restore task behavior by reverting only the selection and Overview empty-state changes in `rust-arr/src/coordinator-workspace.mjs`; preserve all unrelated worktree edits. Redeploy through the same three-file Compose configuration if a rollback is needed.

## Final state

The online metadata node is selected when a saved apex selection cannot provide coordinator workspace data. Overview copy now distinguishes an available but unselected metadata node from no online metadata node.
