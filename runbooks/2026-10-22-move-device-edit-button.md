# Move the device Edit button beside its name

## Objective and initial state

Move the Edit control closer to the device name on coordinator Devices cards. The existing name group expanded across the available heading width, separating Edit from the text input. Work is isolated in the existing `codex/device-name-edit` worktree; the shared checkout has concurrent changes in the same UI area.

## Changes

Changed the device name group and input to use content-sized flex bases, so the Edit button follows the name while the route status remains at the right side of the card heading. Name editing behavior is unchanged. The live Mac mini source was older than the committed UI and lacked the Edit-button code plus device-card layout rules. Backed up its existing coordinator HTML and module, applied only the device-card and edit-button changes, and rebuilt only `roarlay` in Compose project `rawrarr-internal-mini`.

## Verification

`node --check rust-arr/src/coordinator-workspace.mjs`, the focused `node --test scripts/tests/test_device_name_edit_button.mjs` (1/1), and `git diff --check` passed. The focused test confirms Edit remains attached to the name group. After deployment, `roarlay` reported healthy with zero restarts, `https://rawrarr.risk-mermaid.ts.net/health` returned HTTP 200, and Tailscale Serve still mapped `rawrarr.risk-mermaid.ts.net:443` `/` to `http://127.0.0.1:8080`. The authenticated Devices page was not visually checked because browser control could not attach to the Chrome debugger.

## Recovery and remaining work

To roll back the live Mac mini change, restore `coordinator_workspace.html` and `coordinator-workspace.mjs` from `~/Library/Application Support/rawrarr/internal-coordinator/backups/pre-device-edit-20261022-1622/`, then rebuild only `roarlay` under its deployment lock. Reverting the two device-name flex declarations in the repository restores the prior name spacing.
