# Show redacted cluster configuration

## Objective

Display the shared age-encrypted configuration on the coordinator Configuration page while keeping the age private ID and decrypted config inside the user's browser.

## Changes

- The browser fetches config ciphertext through the existing metadata-node relay and decrypts it locally with the browser-held age ID.
- Secret-like config fields, including API keys and the metadata filename key, render as `<REDACTED>`.
- Opening or navigating to the Configuration view loads the config automatically; no load button is required.
- The coordinator backend receives neither the age ID nor plaintext config; its relay continues to carry ciphertext.

## Verification

- `node --test scripts/tests/test_rawrarr_coordinator_workspace.mjs` passed all 8 checks.
- `git diff --check` passed.
- The coordinator backend was not changed to accept age identities or plaintext config; the page uses the existing ciphertext relay endpoint.
- Rebuilt and restarted only `roarlay` in `rawrarr-local` using the active `docker-compose.yml`, `docker-compose.coordinator.yml`, and `docker-compose.tailscale-macbook.yml` files. PostgreSQL and the Tailscale sidecar stayed running.
- The initial Compose invocation omitted the active override files and collided with the Tailscale diagnostics binding on `127.0.0.1:7298`. No existing container was stopped to work around it; rerunning with the active files restored roarlay successfully.
- Roarlay reports healthy; `https://rawrarr.risk-mermaid.ts.net/health` identifies the roarlay service. `tailscale serve status --json` confirms HTTPS `/` proxies to `http://127.0.0.1:8080`; `/direct-api` remains configured to `http://127.0.0.1:8082`.
- Removed the manual load button; opening `#configuration` after sign-in loads the config automatically when the browser has the age ID.

## Recovery

Revert the task-owned changes to `rust-arr/src/coordinator_workspace.html`, `rust-arr/src/coordinator-workspace.mjs`, `rust-arr/web-export/metadata-export.js`, generated `rust-arr/src/assets/metadata-export.js`, the coordinator workspace test, and this runbook/index entry.
