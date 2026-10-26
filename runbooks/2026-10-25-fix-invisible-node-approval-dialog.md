# Fix the invisible pending-node approval dialog

## Objective and tracking

Fix the Devices-page Approve button so the owner signing dialog is visible. GitHub issue [#77](https://github.com/willjasen/rawrarr/issues/77) tracks the fix.

Related procedures: [complete per-instance device signing](2026-10-22-complete-device-signing.md) and [deploy the latest main revision internally](2026-10-25-deploy-latest-main-internal-macs.md).

## Initial state

- The live coordinator page at `https://rawrarr.risk-mermaid.ts.net/#devices` showed one pending MacBook fossil request.
- Clicking Approve set the dialog's `open` state, but it remained invisible with a zero-size bounding box because the dialog was nested in the hidden `#settings-view` section.
- The page console showed no relevant Rawrarr errors. No age identity, encrypted backup, fingerprint, or approval request was entered or submitted.

## Changes and verification

- Moved `#approve-node-dialog` outside `#settings-view`, alongside the workspace sections, so it can enter the browser's top layer when Devices is active.
- Added a regression assertion that the approval dialog stays outside the hidden Settings section and remains opened through `showModal()`.
- Updated the script test index to describe the new regression coverage.
- `node --test scripts/tests/test_rawrarr_coordinator_workspace.mjs` passed all 25 tests. `git diff --check` passed.
- The live coordinator is unchanged; this fix must be published on GitHub `main` and deployed to `roarlay` before the browser page receives it.

## Recovery and remaining work

If the UI change causes a regression, revert only the task-owned dialog placement and regression assertion. Deploy the fix only from a published GitHub `main` revision. The pending MacBook request still requires the owner to provide their age ID, encrypted root backup, and independently saved fingerprint in the visible dialog before signing approval.
