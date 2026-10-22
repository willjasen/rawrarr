# Show the sidebar profile as four words

## Objective and initial state

Render the generated account label beside its avatar as two words on each of two lines. The coordinator sidebar previously concatenated all four generated components and ellipsized the result. Work is isolated in `codex/profile-name-two-by-two`.

Related work: [display the generated account name](2026-10-18-display-generated-account-name.md) and [the coordinator workspace sidebar](2026-10-18-coordinator-workspace-sidebar.md).

## Actions

- Changed the sidebar profile label to render the existing four browser-generated name components in a two-column grid and expose the complete name to assistive technology.
- Updated the coordinator workspace regression check.

## Verification

- `node --test --test-name-pattern='sidebar profile name displays' scripts/tests/test_rawrarr_coordinator_workspace.mjs` passed the new layout regression check.
- `git diff --check` passed.
- The complete existing `test_rawrarr_coordinator_workspace.mjs` run reported 11 passing and 7 failing checks. Its profile assertions include stale expectations that do not match the current workspace source (for example, obsolete loading text); those failures are outside this layout change.
- Built `rawrarr:local` from this task worktree and recreated only `fossil` in the existing `rawrarr-node-internal` project. The MacBook fossil container is healthy with zero restarts; its existing HTTPS Serve `/` and `/direct-api` proxies remain intact, and its `/health` route returned HTTP 200.
- Backed up the Mac mini coordinator's two workspace source files to `~/Library/Application Support/rawrarr/internal-coordinator/backups/pre-sidebar-profile-four-word/`, copied in the updated workspace assets, and rebuilt only `roarlay` in `rawrarr-internal-mini`. Roarlay is healthy with zero restarts. Tailscale Serve still maps `rawrarr.risk-mermaid.ts.net:443` `/` to `http://127.0.0.1:8080`, and the public HTTPS `/health` route returned HTTP 200.
- The `fossil` and `roarlay` deployment locks were claimed and released. Existing app data, PostgreSQL, node volumes, and Tailscale state were preserved.

## Final state

The coordinator sidebar and MacBook Rawrarr node have the four-word two-by-two profile display deployed. No database or account data changed. The full existing workspace test file still has unrelated stale assertions; the added focused layout check passes.
