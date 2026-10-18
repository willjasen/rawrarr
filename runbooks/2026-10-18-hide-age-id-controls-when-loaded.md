# Hide age ID entry controls when loaded

## Objective

Hide the age private ID entry section while a key is saved in this browser profile, while keeping an explicit clear action available.

## Environment and initial state

- Rawrarr coordinator workspace, Movies panel.
- The browser stores the private ID in `localStorage` and synchronizes changes between same-origin tabs.
- The working tree contained unrelated changes before this task; only the workspace UI, its focused test, and this runbook/index entry belong to this task.

## Changes

- Wrapped the age ID label, password field, and unlock action in a dedicated entry section.
- Added visibility synchronization: the entry section is hidden and the clear button is shown while a key exists; clearing or cross-tab sign-out restores the entry section.
- If decryption fails, the entry section becomes visible again so the key can be corrected.
- Added focused source assertions for the visibility state.

## Verification

- `node --test scripts/tests/test_rawrarr_coordinator_workspace.mjs`: 6/6 passed.
- `git diff --check`: passed.
- Rebuilt `rawrarr-local` coordinator; container reports healthy.
- Read back Tailscale Serve configuration: `rawrarr.risk-mermaid.ts.net:443` `/` proxies to `http://127.0.0.1:8080`; `https://rawrarr.risk-mermaid.ts.net/health` returned HTTP 200.
- Reloaded the live Chrome Movies page with the existing browser-profile key. The entry label, input, and Unlock button are absent from the accessibility tree; the separate Clear private ID button is visible. The saved key was not changed.
- The library request was still pending during the visual check, so this task verified the controls and coordinator route, not a completed movie-table refresh.

## Recovery

Revert only the task-owned workspace HTML, module, test, runbook, and runbook index changes if this behavior needs to be rolled back. Do not clear the user's saved browser key during verification.
