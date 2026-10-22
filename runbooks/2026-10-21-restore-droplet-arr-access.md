# Restore Radarr and Sonarr access from the Droplet coordinator

## Objective and initial state

Restore the established coordinator workspace flow so a browser can connect to Radarr and Sonarr on an online user-deployed metadata node while roarlay runs on the DigitalOcean Droplet. The main checkout was clean at `7cb10e2`; existing Copilot worktrees remain untouched. The Droplet migration worktree advertises neither relay nor direct node targets (`RAWARR_NODE_RELAY_TARGETS: "{}"`, no `RAWARR_NODE_DIRECT_TARGETS`). The browser peer access work is a separate, unmerged feature and is not required for direct browser access.

## Actions

- Changed the coordinator workspace to use the configured user-node URL directly for Arr API operations and settings. API keys and direct session credentials stay in browser-to-node requests; they are not sent through coordinator relay.
- Require online status plus a configured direct URL or relay before enabling node selection. Normal route discovery remains for other workspace functions.
- Generalized direct-target validation to allow separate tailnet HTTPS origins (including per-user tailnet domains), with safe path, no port, no credentials, no query/fragment, and different-origin checks.
- Updated focused regression assertions.

## Verification

- Focused workspace test for direct Radarr/Sonarr access passes.
- `node --check rust-arr/src/coordinator-workspace.mjs` and `git diff --check` pass.
- The full workspace test file still contains pre-existing failures against stale UI assertions (TV status markup, profile image implementation, and route label wording); it was not clean on the unchanged baseline.
- No deployment or live Radarr/Sonarr request was made. The Droplet Compose needs a per-account direct URL target, and each user node must allow `https://rawrarr.willjasen.net` in its exact-origin CORS/session exchange.

## Recovery and remaining work

Revert the workspace direct URL change and the `validate_direct_target` change to restore the previous coordinator-origin/specific-tailnet allow rules. Preserve the existing browser-access branches. Complete the Droplet configuration with approved node IDs and their per-account tailnet `/direct-api/` URLs, then validate CORS/session setup and real browser requests to Radarr and Sonarr before claiming the user flow is restored.
