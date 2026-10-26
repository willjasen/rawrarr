# Move Arr connection setup to Movies and TV

## Objective and issue

Move Radarr setup from Settings to Movies and Sonarr setup from Settings to TV. Each page must request its URL and API key when missing and keep its library details hidden until configured.

GitHub issue creation was attempted with the authenticated `gh` CLI, but GitHub API access failed with `error connecting to api.github.com`. Proposed issue: “Move Radarr and Sonarr setup into their library pages.” Acceptance criteria: remove Arr connection forms from Settings; present each form on its corresponding page when unconfigured; do not fetch or display library details until configured; preserve encrypted node-side credential storage and hidden API-key behavior.

## Changes

- Relocated the existing encrypted connection forms into Movies and TV.
- Added configured-state gating so library requests and visible details wait until each service is connected.
- Updated the Sonarr browser regression test and test index for the page-local setup flow.
- Updated the user guide to direct users to the Movies and TV pages.

## Verification

- `node --check scripts/tests/test_rawrarr_sonarr_settings.mjs` passed.
- `git diff --check` passed.
- `node --test scripts/tests/test_rawrarr_sonarr_settings.mjs` could not start because `/usr/local/lib/node_modules/playwright-core/index.mjs` is missing in this environment.

## Final state

Work is in the dedicated `codex/move-arr-settings` branch and its worktree. No push or deployment was requested.
