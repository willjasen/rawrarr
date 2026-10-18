# Show node online and offline status in the coordinator

## Objective and initial state

Show a clear green online or red offline status for each connected metadata or storage device on the coordinator workspace. The node list already refreshed every 10 seconds and exposed an `online` boolean, but the UI buried online state in gray detail text and described offline devices only as last seen earlier.

## Actions

- Added an explicit Online/Offline label and colored indicator to each coordinator node button, with a non-color text label and accessible name.
- Preserved the existing last-seen and relay-route details; offline nodes remain selectable and status continues refreshing with the existing node-list poll.
- Added focused source assertions for online/offline state and colors, and updated the test index.

## Verification

- Pending.

## Final state and remaining work

- Pending.
