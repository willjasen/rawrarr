# Add an About page

## Objective

Add a page that introduces rawrarr and explains its current role, media integrations, encrypted metadata synchronization, and future file-sharing direction.

## Initial state

The coordinator at `https://rawrarr.risk-mermaid.ts.net/` serves its own workspace in `rust-arr/src/coordinator_workspace.html`, with navigation state in `rust-arr/src/coordinator-workspace.mjs`. The first implementation mistakenly changed the node dashboard in `rust-arr/src/ui.html`; that page is not used by the coordinator route.

## Actions

- Added an About navigation entry directly below Settings in the coordinator sidebar and an `#about` view in the coordinator workspace.
- Kept the earlier node dashboard About view, but the deployed tailnet page uses the coordinator workspace.
- Described current media-library and metadata behavior, distinguishing it from future general-purpose file sharing.
- Added `scripts/tests/test_rawrarr_navigation.mjs` as the focused check to run below.

## Verification

- Static checks confirmed that About follows Settings in the coordinator sidebar, `#about` is accepted by the coordinator view map, and its page is present.
- `git diff --check` passed for the relevant files.
- `scripts/tests/test_rawrarr_navigation.mjs` could not run because its hard-coded Playwright module at `/usr/local/lib/node_modules/playwright-core/index.mjs` is missing.

## Deployment and verification

- Atomically claimed `~/Library/Application Support/Rawrarr/.container-rebuild-roarlay.lock` after confirming it was absent; verified port 7297 was owned by OrbStack for the existing Rawrarr/Tailscale deployment.
- The documented single-file Compose command failed to start roarlay because base `docker-compose.yml` also published 7298 while the running Tailscale container owned that port. Rebuilt with the existing `docker-compose.tailscale-macbook.yml` override, which removes roarlay's host ports and preserves the shared Tailscale namespace.
- `rawrarr-local-roarlay-1` is running and healthy on the newly built image. Local `http://127.0.0.1:7298/health` returned the roarlay health page. The user then reported About was not visible; investigation showed the root route is the separate coordinator workspace and the initial change was in the node UI. The coordinator workspace has now been updated to include About.
- Read back Tailscale Serve JSON: `rawrarr.risk-mermaid.ts.net:443` `/` proxies to `http://127.0.0.1:8080`. `https://rawrarr.risk-mermaid.ts.net/health` returned the roarlay health page.
- After the user reported the About entry was missing, deployed the coordinator workspace fix to roarlay with the same override. Verified the served `/assets/coordinator-workspace.mjs` contains the About route, roarlay reports healthy, HTTPS health succeeds, and Serve still targets `http://127.0.0.1:8080`. No metadata or storage service was rebuilt.
- Removed the deployment lock after verification.

## Remaining work

- The Playwright navigation check remains unverified because its configured Playwright module is missing from this environment.
- No commit was made because the UI file and runbook index contain extensive pre-existing edits that make task ownership unclear.
