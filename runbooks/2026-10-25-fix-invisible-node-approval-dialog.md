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
- PR [#79](https://github.com/willjasen/rawrarr/pull/79) passed the required `Basic script tests` check and merged as `0b81fb3b14ffe89c53342e73b2217ed52d6faff4`. The deployment classifier selected the coordinator only.
- Created and verified a protected PostgreSQL dump at `~/Library/Application Support/Rawrarr/backups/pre-approval-ui-0b81fb3-20261025/coordinator.dump`. Built from an isolated detached checkout of that exact published `main` revision, tagged the prior app image `rawrarr:rollback-roarlay-before-0b81fb3`, and recreated only `roarlay` with `--no-deps`. New image ID: `sha256:038389a4b4a784ff4c8550f549c08a5dabeca496689e15236362834e26528a94`.
- Roarlay is healthy with zero restarts. PostgreSQL remained healthy and accepting connections; the Tailscale sidecar remained running. Serve readback still maps `rawrarr.risk-mermaid.ts.net:443` `/` to `http://127.0.0.1:8080`, and HTTPS `/health` returned HTTP 200.
- Reloaded the live Devices page and clicked Approve. The dialog opened visibly at 640×538 pixels as a direct child of `MAIN`. Closed it without entering the age ID, root backup, or fingerprint and without submitting an approval. The deployment lock was released and confirmed absent.
- Closed issue [#77](https://github.com/willjasen/rawrarr/issues/77) after deployment and live UI verification.

## Recovery and remaining work

If the UI change causes a regression, revert only the task-owned dialog placement and regression assertion. Restore the prior `roarlay` image tag while holding its deployment lock. The pending MacBook request still requires the owner to provide their age ID, encrypted root backup, and independently saved fingerprint in the visible dialog before signing approval; no approval was submitted by this task.
