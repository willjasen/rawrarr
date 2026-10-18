# Improve the device topology diagram

## Objective
Replace the device/peer rows with a simple network diagram that draws one line for every device pair.

## Initial state
The topology listed devices separately and rendered only peer links reported by Syncthing. The repository already has extensive unrelated changes; this task is limited to the coordinator topology markup, its focused test, and this runbook/index entry.

## Actions
Render the enrolled nodes around a diagram and draw every unique pair. Color lines green when a matching peer report is connected, red when reported disconnected, and neutral when live peer state is unavailable. Keep node role and online state visible, and explain the line states in a legend.

## Verification
`node --test scripts/tests/test_rawrarr_coordinator_workspace.mjs` passed (13/13), `node --check rust-arr/src/coordinator-workspace.mjs` passed, and `git diff --check` was clean for task files. The updated `rawrarr-local` roarlay image was built and deployed using the existing base, coordinator, and MacBook Tailscale Compose files. Roarlay is healthy; local and tailnet `/health` return HTTP 200; Tailscale Serve still maps `rawrarr.risk-mermaid.ts.net:443` `/` to `http://127.0.0.1:8080`. The authenticated visual Devices page was not independently inspected in a browser.

The initial Compose attempt used only the base file and failed because roarlay then requested 127.0.0.1:7298, already published by the Tailscale sidecar. The existing deployment metadata identified the three-file Compose project; rerunning with those files restored roarlay and completed deployment without changing the sidecar.

## Final state
Topology renders nodes around a simple network diagram and draws one line per unique device pair, so three nodes produce three lines. Connected peer reports are green, reported-disconnected peers red, and pairs without a live peer report neutral. The changes remain uncommitted: affected source and test files are part of the extensive pre-existing untracked/modified worktree, so a task-only commit cannot be isolated safely from this task's patch.
