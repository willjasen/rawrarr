# Move the device delete button to the card corner

## Objective and initial state

Move the coordinator Devices view's Delete device action to the bottom-right of its device card. The shared main checkout already contained unrelated edits in `rust-arr/src/coordinator.rs` and a coordinator workspace test, plus an untracked restore-access runbook; those files are outside this task.

## Changes

Made device cards vertical flex containers, placed Delete device after health and any storage-scan content, and aligned it to the card's right edge. Existing button behavior and accessibility label are unchanged.

## Verification

`node --check rust-arr/src/coordinator-workspace.mjs` and `git diff --check` passed. This commit records the coordinator UI change; it has not been deployed.

## Recovery and remaining work

Revert the task-owned changes in `rust-arr/src/coordinator_workspace.html` and `rust-arr/src/coordinator-workspace.mjs` to restore the previous button position. This is a coordinator UI change and has not been deployed.
