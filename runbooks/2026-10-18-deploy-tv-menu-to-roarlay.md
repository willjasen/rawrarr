# Deploy the TV menu label to roarlay

## Objective

Correct the coordinator sidebar and active page heading from “Series” to “TV” for Sonarr.

## Initial state

- The previous menu edit changed the main app UI in `rust-arr/src/ui.html`, but the live roarlay coordinator uses `rust-arr/src/coordinator_workspace.html`.
- The live roarlay container was healthy and the tailnet Serve route pointed to `http://127.0.0.1:8080`.
- The roarlay deployment lock was absent before this task.

## Actions

- Updated the coordinator sidebar label, page title, and unavailable-library message to use “TV”.
- Added a static regression test for the coordinator menu label and page heading.

## Verification

- `node --test scripts/tests/test_rawrarr_coordinator_workspace.mjs` passed all 9 checks; `node --check` and `git diff --check` passed.
- The first rebuild used only the default Compose file and failed because its roarlay port mapping collided with the existing Tailscale sidecar on 127.0.0.1:7298. Inspected the running sidecar metadata and retried with the existing project files: `docker-compose.yml`, `docker-compose.coordinator.yml`, and `docker-compose.tailscale-macbook.yml`.
- The `rawrarr-local` roarlay container is healthy. Local and tailnet `/health` both returned HTTP 200 and identified the Roarlay web service.
- `tailscale serve status --json` confirms `rawrarr.risk-mermaid.ts.net:443` `/` proxies to `http://127.0.0.1:8080`; the existing `/direct-api` handler remains present.
- Refreshed the already-open authenticated Chrome tab. Its sidebar visibly shows “TV” and the existing `#devices` page state is preserved.

## Final state

- The coordinator menu and Sonarr section heading use “TV” in source and in the refreshed live UI.
