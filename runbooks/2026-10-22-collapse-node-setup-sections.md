# Collapse completed node setup sections

## Objective and starting state

Compact completed age ID and signing-root sections on `/setup/node`, reveal the next gated section, and offer a one-click copy of the generated Compose file. The task started from the `main` branch in an isolated worktree. The shared checkout had unrelated uncommitted work and was left unchanged. Existing onboarding behavior is documented in [subscriber node onboarding](2026-10-18-subscriber-node-onboarding.md).

## Work and verification

- Added compact summaries and Edit controls for completed sections. Age and root completion are derived from the existing backup gates; deployment can be marked complete after preparing the Compose file.
- Added a `Copy compose.yml to clipboard` action and changed the downloaded filename to `compose.yml`. Download now leaves the generated preview in place for copying.
- Extended the browser test for section collapse and clipboard content and updated the test index.
- `node --check rust-arr/src/assets/subscriber-onboarding.js` and `git diff --check` passed. The browser test could not start because `/usr/local/lib/node_modules/playwright-core/index.mjs` is unavailable in this environment.
- Deployment and live route verification were not performed: Docker inspection found the requested `rawrarr-local` Compose project was not present. The only running Rawrarr app container belongs to `rawrarr-orbstack-test`, so it was left untouched. The documented protected password file exists, but starting a new `rawrarr-local` project would violate the requirement to update the existing deployment only.

## Recovery and remaining work

Revert this task's UI, browser-test, test-index, and runbook changes, then redeploy roarlay if deployment has occurred. When the existing `rawrarr-local` deployment is available, deploy under its documented roarlay Compose lock and verify roarlay health and the tailnet HTTPS route before reporting the UI change live.
