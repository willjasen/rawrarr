# Show peer counts and device topology

## Objective
Show peer connectivity counts on every device card and add a topology section below the device list.

## Initial state
The MacBook card used folder-level health and showed `1/1 peers connected`, while the Mac mini card used a matched-peer branch that omitted peer counts. The Devices page had no topology section. The repository contains extensive unrelated user changes; this task is limited to the coordinator workspace UI, focused test, this runbook, and its index entry.

## Changes
The matched-peer status now includes its source folder's connected/total peer count. Added a responsive Topology section showing enrolled devices and links derived from uniquely matched Syncthing peer reports, including connection state and completion. Folder IDs remain hidden from user-facing labels.

## Verification
`node --test scripts/tests/test_rawrarr_coordinator_workspace.mjs` passed (17/17), `node --check rust-arr/src/coordinator-workspace.mjs` passed, and `git diff --check` was clean for task files. Roarlay is healthy; local and tailnet `/health` return HTTP 200; Tailscale Serve still maps `/` to `http://127.0.0.1:8080`. The live Devices page shows the Topology section and both enrolled devices. Peer health currently reports unavailable, so live peer counts and links could not be verified. Inspection found that roarlay's configured MacBook metadata target `127.0.0.1:8081` has no listener in its network namespace; the metadata container has a separate network namespace. The Mac mini `/health` route does respond over the tailnet. No metadata or storage container was redeployed for this roarlay UI task.

## Final state
The matched-peer card path now includes the source folder's connected/total peer count. The Devices page has a responsive topology section with devices and uniquely matched Syncthing peer links. Live peer data remains unavailable until the existing MacBook metadata relay target is reachable from roarlay.
