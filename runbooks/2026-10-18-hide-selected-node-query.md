# Keep the selected coordinator node out of the browser URL

## Objective and initial state

The coordinator stored its selected node in `?node=...`, making internal UI state visible in the browser address bar. Preserve the selected node when the page refreshes without exposing it in the URL.

Related: [Preserve the selected node page on refresh](2026-10-18-preserve-coordinator-node-on-refresh.md).

## Actions

- Store the selected node in per-tab `sessionStorage` and restore it after refresh.
- Continue accepting a legacy `node` query parameter, remove it from the address bar, and keep node navigation query-free.
- Update the coordinator workspace regression assertions.

## Verification

- `node --test scripts/tests/test_rawrarr_coordinator_workspace.mjs`: 6 passed.
- `git diff --check` passed.
- Rebuilt only the `coordinator` service in the existing `rawrarr-local` Compose project. The coordinator reports healthy; local and tailnet `/health` return HTTP 200.
- Tailscale Serve read-back confirms `rawrarr.risk-mermaid.ts.net:443` `/` still proxies to `http://127.0.0.1:8080`; the `/direct-api` handler remains present.
- The deployment marker was removed after verification. An authenticated browser refresh was not exercised.

## Final state

Selected-node persistence now uses per-tab `sessionStorage`; opening a legacy `?node=...` URL removes that query from the address bar, and selecting a node does not add it back. Existing unrelated worktree changes remain untouched and uncommitted.

## Recovery

Revert only the selected-node `sessionStorage` and URL cleanup changes in `rust-arr/src/coordinator-workspace.mjs` and the corresponding regression assertions.
