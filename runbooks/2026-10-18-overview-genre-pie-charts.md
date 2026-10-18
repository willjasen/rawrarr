# Overview genre pie charts

## Objective

Show Radarr and Sonarr genre distributions as larger pie charts on Overview, listing up to ten genres and combining all remaining genres into an `Other` slice.

## Initial state

Overview showed genre counts as inline text. The coordinator workspace files are untracked in a checkout with extensive pre-existing changes; preserve that ownership and limit this task to the genre UI, a focused regression assertion, and this runbook/index entry.

## Actions

- Replaced the text summary with an accessible conic-gradient pie chart and matching color legend.
- Sorted genres by frequency and name, counted each library record once per distinct genre, kept the top ten, and aggregated counts beyond ten into `Other` only when needed.
- Expanded both genre cards across two grid columns with a larger chart area and responsive mobile layout.

## Verification

`node --check rust-arr/src/coordinator-workspace.mjs` and scoped `git diff --check` passed. `node --test scripts/tests/test_rawrarr_coordinator_workspace.mjs` ran the new genre chart assertion successfully (13/17 passed); four existing assertions failed because the current untracked coordinator workspace differs from the pre-existing test expectations for route status and coordinator markup/script behavior. Deployed `roarlay` in Compose project `rawrarr-local` using the existing protected password-file path. The container reached healthy status, local `http://127.0.0.1:7297/health` and tailnet `https://rawrarr.risk-mermaid.ts.net/health` both returned HTTP 200 identifying Roarlay, and Tailscale Serve JSON confirmed `/` still proxies to `http://127.0.0.1:8080` with `/direct-api` retained. The authenticated Overview itself was not browser-checked. The deployment marker was removed after verification.

## Recovery

Revert the task-owned changes in `rust-arr/src/coordinator_workspace.html`, `rust-arr/src/coordinator-workspace.mjs`, `scripts/tests/test_rawrarr_coordinator_workspace.mjs`, this runbook, and its `runbooks/README.md` entry. No deployment was performed.
