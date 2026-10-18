# Hide coordinator passkey migration on the tailnet

## Objective and initial state

Keep the coordinator-origin passkey migration action available on the local metadata origin (`http://localhost:7297`) and hide it on other origins, including `https://rawrarr.risk-mermaid.ts.net/`. The action is only needed to migrate a local account's passkey to the coordinator origin. The checkout contained extensive unrelated user changes; only the dashboard, age setup page, focused source test, this runbook, and its index entry were edited for this task.

## Actions

- Made the dashboard button hidden by default and reveal it only at the exact local origin.
- Hid the age setup migration section by default and reveal it only at the exact local origin.
- Added focused source assertions for the origin gate.

## Verification

- Source review confirms both views use the exact `http://localhost:7297` origin check.
- Automated tests were not run.
- Rebuilt only the `metadata` service in Compose project `rawrarr-local` while holding `.container-rebuild-metadata.lock`; the lock was removed after verification.
- The metadata container is `running` and `healthy`; `http://127.0.0.1:7297/health` returned successfully and `/` redirects to `/auth` when signed out.
- `https://rawrarr.risk-mermaid.ts.net/health` responds with the coordinator health page, confirming the tailnet route still reaches the coordinator.
- `git diff --check` passed for task-owned files. Automated tests were not run.

## Recovery and remaining work

Revert only the task-owned UI and test edits to restore the prior behavior. Deployment follows the project-local container marker procedure for the `metadata` service.
