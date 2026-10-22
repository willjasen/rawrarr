# Render cached Movies and TV text immediately

## Objective and initial state

Movies and TV waited for direct node requests before building their tables, even when the browser had loaded the same library before. The shared checkout has unrelated coordinator, runbook index, test, and restore-runbook changes; preserve those. Related context: [temporary direct Radarr library route](2026-10-22-temporary-direct-radarr-library.md) and [Sonarr library timeout fix](2026-10-22-fix-sonarr-library-timeout.md).

## Changes

- Save successful Movies and TV text responses in browser `localStorage`, scoped by node ID and media type. Cache records contain library fields only, not credentials, private identities, or image bytes.
- Render saved text immediately when opening Movies or TV, then refresh it from the selected node. A failed refresh leaves already displayed Movies rows available.
- Attach lazy artwork URLs after rows render; artwork routing or image delays no longer block the text table.
- Remove the corresponding text cache after a user-requested metadata rescan; retain it when the age ID is cleared.

## Verification

- `node --check rust-arr/src/coordinator-workspace.mjs` passed.
- `node --test scripts/tests/test_rawrarr_library_text_cache.mjs` passed (2 tests).
- Live browser behavior and deployment were not verified in this task.

## Recovery

Revert the task-owned cache/render changes in `rust-arr/src/coordinator-workspace.mjs` and remove this test and runbook entry to restore network-first rendering.

## Deployment

- Deployed commit `a13a2ce` from a clean Git archive to the Mac mini coordinator source and the MacBook node build source; no unrelated checkout changes were sent.
- Rebuilt and recreated only `roarlay` in `rawrarr-internal-mini` on `brandons-mac-mini`, and only `fossil` in `rawrarr-node-internal` on `brandons-macbook-pro`. PostgreSQL, Tailscale sidecars, and their volumes were preserved.
- The coordinator's deployed `/assets/coordinator-workspace.mjs` SHA-256 matched the committed source. `https://rawrarr.risk-mermaid.ts.net/health` returned HTTP 200; Tailscale Serve still maps HTTPS `/` to `http://127.0.0.1:8080`.
- Mac mini roarlay and PostgreSQL are healthy with zero restarts. MacBook fossil is healthy with zero restarts and its local `/health` responds successfully.
- Deployment locks for the coordinator and fossil were removed and verified absent after checks. The authenticated Movies/TV UI was not opened or refreshed, so browser-visible behavior remains unverified.
