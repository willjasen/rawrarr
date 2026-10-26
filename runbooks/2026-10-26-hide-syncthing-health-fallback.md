# Hide unavailable Syncthing health until configured

## Objective

Remove the “Syncthing health unavailable” placeholder from device cards while subscriber Syncthing API reporting remains unconfigured. This is a temporary UI change related to issue [#142](https://github.com/willjasen/rawrarr/issues/142).

## Initial state

The current Devices UI displayed that placeholder whenever the report was absent or returned `configured: false`. The API remains unconfigured in the internal subscriber containers, as recorded in [the investigation runbook](2026-10-26-investigate-syncthing-health-configuration.md). Work started on a branch based on fetched `origin/main` commit `e530450cd1e364beb140c0ddf51b03e33cf3efb5`.

## Changes

- Hide the Syncthing status element when its report is unconfigured.
- Add a focused source regression asserting the placeholder is absent and the element is hidden in that branch.
- Update the script-test index.

## Verification

- `node --test scripts/tests/test_rawrarr_coordinator_workspace.mjs` — passed, 28/28.
- `git diff --check` — passed.
- Reviewed the task diff; only the requested UI fallback, its regression assertion/index entry, and this runbook/index are changed.

## Final state and remaining work

The UI change is local to this task branch. Issue #142 continues to track the safe subscriber-to-host Syncthing health connection and full live verification.
