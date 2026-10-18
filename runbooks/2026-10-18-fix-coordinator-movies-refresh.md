# Fix coordinator movie library clearing during node polling

## Objective and initial state

The signed-in coordinator Movies page was reported to refresh and clear repeatedly. The existing page polls nodes every 10 seconds; the coordinator DOM held 1,941 decrypted movies during observation. The checkout contains extensive unrelated user changes, including edits to adjacent Rawrarr files; keep this task scoped and uncommitted if ownership overlaps.

Related: [Show connected node pages from the coordinator](2026-10-18-coordinator-node-workspace.md), [Show the full Movies page through the coordinator](2026-10-18-full-coordinator-movies-page.md).

## Actions

Updated `loadNodes()` so its 10-second status poll preserves the currently selected node and movie table. It updates node details and pressed state without calling `openNode()` again. If the selected node disappears or loses its relay, it clears the selection; initial load still restores the URL-selected node.

Added focused regression assertions to `scripts/tests/test_rawrarr_coordinator_workspace.mjs`.

## Verification

- Reproduced the issue in the existing Chrome tab: during one 10-second poll interval, the movie table showed 0 rows and “Fetching encrypted Radarr library…” before returning to 1,941 rows.
- `node --test scripts/tests/test_rawrarr_coordinator_workspace.mjs` passed (6/6).
- Once the pre-existing coordinator and metadata deployment locks cleared, claimed the coordinator marker atomically and rebuilt only the coordinator service in project `rawrarr-local` using the Compose file set read from the existing container labels. The protected PostgreSQL password file was passed by path; its contents were not read. The task marker was removed after verification.
- Local `http://127.0.0.1:7298/health` and public `https://rawrarr.risk-mermaid.ts.net/health` both returned HTTP 200. Serve read-back confirms HTTPS 443 `/` → `http://127.0.0.1:8080`; the existing `/direct-api` → `http://127.0.0.1:8082` handler remains present.
- Reloaded the existing Chrome tab. The Movies page rendered 1,941 rows and remained at 1,941 after the next 10-second node polling interval, with `macbook-metadata` selected and the Movies URL preserved.
- `node --test scripts/tests/test_rawrarr_coordinator_workspace.mjs` passed (6/6).
- The coordinator workspace source and test files were already untracked in the initial checkout, and the runbook index was already modified. These files contain overlapping pre-existing work, so changes are left uncommitted.

## Recovery

Revert only this task's polling/render preservation changes and its regression coverage.
