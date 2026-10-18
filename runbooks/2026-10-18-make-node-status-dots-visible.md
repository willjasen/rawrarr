# Make coordinator node status dots visible

## Objective and initial state

Make the online/offline state of each node easy to see in the coordinator workspace. The live page's accessibility tree already exposed Online labels and source markup contained green/red indicator spans, but the 9-pixel dots were too subtle to notice.

## Actions

- Increased each status indicator to 13 pixels, added a clear border and a wider colored halo, while retaining the text label and accessible status.
- Added focused assertions for the indicator dimensions and green/red styling.

## Verification

- The coordinator rebuilt and restarted healthy in the existing `rawrarr-local` project. Local metadata remained healthy; `http://127.0.0.1:7297/health`, coordinator HTTPS health, and the coordinator-to-Mini relay health all returned HTTP 200.
- Inspected the live coordinator workspace screenshot after rebuild: both `macbook-metadata` and `mac-mini-storage` visibly show green dots beside their Online labels. Both devices are online, so no red/offline dot is expected in this view.
- The focused coordinator workspace tests passed (5/5), Compose validation passed, and `git diff --check` passed.

## Recovery and remaining work

- Revert only the status-indicator rules in `rust-arr/src/coordinator_workspace.html` and their focused assertions if needed.
- No further deployment is needed for this change.
