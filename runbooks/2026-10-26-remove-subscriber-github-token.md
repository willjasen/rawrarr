# Remove coordinator GitHub token from subscriber Compose

## Objective and initial state

Fix the subscriber Compose interpolation failure where `docker compose up -d` required `RAWARR_UI_GITHUB_TOKEN_PATH`. Only the coordinator entrypoint runs the private GitHub UI updater; subscribers should not receive its GitHub token. This is the remaining-token finding in [issue #34](https://github.com/willjasen/rawrarr/issues/34).

Relevant earlier work: [defer subscriber Compose deployment work](2026-10-25-defer-subscriber-compose.md).

## Actions

- Removed the private UI updater token environment, secret mount, and required host path from generated subscriber Compose. Headscale auth-key file handling remains conditional on public Headscale mode.
- Updated the subscriber Compose tests and rebuilt the browser bundle embedded in roarlay.

## Verification

`node --test scripts/tests/test_rawrarr_subscriber_compose.mjs` passed 3/3. Generated internal fossil and Headscale boneyard Compose files both passed `docker --context orbstack compose ... config --quiet` with `RAWARR_UI_GITHUB_TOKEN_PATH` explicitly unset. `git diff --check` passed. No node container was started or deployed.

## Recovery and remaining work

Revert the task-owned Compose template, embedded asset, test, and runbook changes to restore the required GitHub token path. Other subscriber installer gaps in issue #34 remain separate.
