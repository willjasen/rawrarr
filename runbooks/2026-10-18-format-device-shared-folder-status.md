# Format device shared-folder status

## Objective
Make the coordinator Devices page show concise shared-folder health without exposing the Syncthing folder ID.

## Initial state
The coordinator status included the internal `rawrarr-shared` folder ID in its displayed text. The repository already contained extensive unrelated user changes; this task is limited to the coordinator workspace UI, its focused test, this runbook, and the runbook index.

## Changes
Removed folder IDs from matched-peer and folder-summary status text. Shared-folder status is rendered as separately styled state and detail items, including completion and peer connectivity when available.

## Verification
`node --test scripts/tests/test_rawrarr_coordinator_workspace.mjs` passed (14/14), `node --check rust-arr/src/coordinator-workspace.mjs` passed, and `git diff --check` was clean for task files. The first deployment attempt omitted the Compose override files recorded on the running containers and could not bind port 7298 because the Tailscale sidecar owns that port. Retried with the existing `docker-compose.yml`, `docker-compose.coordinator.yml`, and `docker-compose.tailscale-macbook.yml` configuration; roarlay is healthy, local `/health` returns HTTP 200, Serve `/` still proxies to `http://127.0.0.1:8080`, and the tailnet `/health` returns HTTP 200.

## Final state
The coordinator Devices page now omits the internal folder ID and styles state and details separately. Deployment and route verification passed. The deployment lock was released.
