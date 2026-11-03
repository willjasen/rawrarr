# Hide unconfigured Arr services from Overview

## Objective and initial state

Hide Radarr and Sonarr information on Overview until each service is configured on the selected metadata node. Issue [#157](https://github.com/willjasen/rawrarr/issues/157).

The Overview rendered both services' summary cards and genre panels before settings were loaded. Settings loading already returned a per-service `configured` flag.

## Changes

- Added initially hidden Radarr and Sonarr summary and genre cards.
- The service settings loader reveals only that service's Overview cards after a successful configured response. Cards remain hidden while refreshing and when the settings request fails.
- Added regression assertions for hidden initial markup and settings-driven visibility.

## Verification

- `node --test scripts/tests/test_rawrarr_coordinator_workspace.mjs` passed (28 tests).
- `git diff --check` passed.
- Live browser verification was not run.

## Final state

The four Arr Overview cards now remain hidden until their matching service is configured on the selected node. Changes are on `codex/hide-unconfigured-arr-overview` and ready for review; no deployment or push was requested.
