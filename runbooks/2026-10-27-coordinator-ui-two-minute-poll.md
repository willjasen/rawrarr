# Coordinator UI bundle checks every two minutes

## Objective

Have the coordinator check the latest private GitHub UI release every two minutes so published UI changes are detected sooner.

## Issue tracking

GitHub issue creation was attempted with the authenticated `gh` CLI but the execution environment could not connect to `api.github.com`. Proposed issue:

- Title: Check coordinator UI releases every two minutes
- Objective: Change the coordinator UI bundle updater to check GitHub every two minutes.
- Acceptance criteria: The coordinator updater defaults to a 120-second check interval; its minimum is 120 seconds; focused regression coverage and the UI distribution runbook describe the cadence; subscriber behavior and runtime data remain unchanged.

## Changes

- Changed the updater default and minimum interval to 120 seconds.
- Updated interval regression coverage, the test index, and deployment guidance.
- No existing-state, data, or protocol migration is needed. The cadence is part of the application image and takes effect when the coordinator image is next updated. Rollback is the previous image or restoring the prior interval values in a subsequent image.

## Verification

- `python3 scripts/tests/test_ui_bundle.py -v` — passed (6 tests).
- Live coordinator update was not performed.
