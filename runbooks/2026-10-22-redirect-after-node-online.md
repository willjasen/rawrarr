# Open Devices when the subscriber node connects

## Objective and starting state

After generating a subscriber Compose file, show an online-wait indicator and automatically take the owner to Devices as soon as that exact node requests enrollment or is already online. The preceding node setup change is recorded in [collapse completed node setup sections](2026-10-22-collapse-node-setup-sections.md).

## Work and verification

- The onboarding page polls the authenticated `/api/workspace/nodes` and `/api/workspace/pending` endpoints without caching and matches the generated node ID. It keeps a spinner and “Waiting for node to come online” message until that node is online or its enrollment request appears. New nodes must be approved before they can report as online, so detecting the pending request returns the owner to Devices for approval.
- Once connected or online, it displays a check and automatically navigates to `/#devices`. The coordinator workspace reads the `#devices` hash and opens Devices; `/app` is not a route in this application.
- Extended the onboarding browser test to simulate no enrollment followed by a pending enrollment request and verify the automatic Devices navigation when the node appears in pending approvals. `node --check rust-arr/src/assets/subscriber-onboarding.js` and `git diff --check` passed. The browser test cannot start because the configured Playwright module is unavailable in this environment.
- Live deployment remains unavailable because this environment has no existing `rawrarr-local` Compose project. Do not create a replacement project for verification.

## Recovery and remaining work

Revert this task's onboarding UI, browser test, test index, and runbook changes to remove the automatic wait and redirect. Deploy only when the documented persistent `rawrarr-local` project is available, then verify the browser flow against an authenticated account and an approved node that checks in.
