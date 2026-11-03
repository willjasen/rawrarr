# Align the Devices add-device button

## Objective and initial state

Rename the coordinator Devices page action to “Add a device” and place it at the right side of the page heading. The action currently appears below the heading as “Add a device with Docker Compose”; the first-node empty state has a separate setup link that should remain unchanged. Tracked by [issue #152](https://github.com/willjasen/rawrarr/issues/152).

Started from fetched `origin/main` (`a854611`) in the dedicated worktree `/Users/willjasen/.codex/worktrees/devices-add-device-button`, branch `codex/devices-add-device-button`.

Relevant prior context: [simplify first-account onboarding](2026-10-22-simplify-first-account-onboarding.md) established the Devices page setup entry point and empty state.

## Actions

- Replaced the standalone action paragraph with a flex heading row containing the Devices title and “Add a device” link to `/setup/node`.
- Made the heading row wrap on narrow screens and align the action to the right.
- Added source regression coverage for the label, link, layout rules, and unchanged first-node setup link.

## Verification

`node --test scripts/tests/test_rawrarr_devices_add_button.mjs` passed (1/1). `git diff --check` passed. No browser render or live deployment was performed.

## Final state and remaining work

The change is ready for pull request review on the task branch. No deployment is part of this task.
