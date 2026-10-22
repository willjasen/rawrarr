# Move the device Edit button beside its name

## Objective and initial state

Move the Edit control closer to the device name on coordinator Devices cards. The existing name group expanded across the available heading width, separating Edit from the text input. Work is isolated in the existing `codex/device-name-edit` worktree; the shared checkout has concurrent changes in the same UI area.

## Changes

Changed the device name group and input to use content-sized flex bases, so the Edit button follows the name while the route status remains at the right side of the card heading. Name editing behavior is unchanged.

## Verification

`node --check rust-arr/src/coordinator-workspace.mjs`, the focused `node --test scripts/tests/test_device_name_edit_button.mjs` (1/1), and `git diff --check` passed. The focused test confirms Edit remains attached to the name group; no browser screenshot verification was performed.

## Recovery and remaining work

Revert the two device-name flex declarations in `rust-arr/src/coordinator_workspace.html` to restore the previous spacing.
