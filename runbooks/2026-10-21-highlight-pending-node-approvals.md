# Highlight pending node approvals

## Objective and initial state

Make new node approvals easy to notice in the coordinator Devices view. The pending approvals section was below the connected-node list and had only a divider, which left it out of view until scrolling.

## Changes

- Move pending approvals to the top of the Devices panel, immediately below its heading.
- Give the section a soft amber background and border while requests are pending, with a `Needs review` badge that is hidden when the queue is empty.
- Apply the same first-position treatment to the authenticated coordinator workspace, which has its own Devices renderer.
- Keep pending requests and approval controls unchanged.

## Verification

- `node --test scripts/tests/test_rawrarr_node_approval_ui.mjs` passed all four focused tests; `git diff --check` passed.
- Rebuilt only `roarlay` in Compose project `rawrarr-orbstack-test`. It reports healthy. Local health at `http://127.0.0.1:17397/health` and tailnet health at `https://rawrarr-orbstack-test-coordinator.risk-mermaid.ts.net/health` both return HTTP 200. A first request during startup returned an empty reply; retries after the health transition passed.
- Refreshed the authenticated Devices page with a cache-busting query. Browser accessibility state shows the pending section immediately after the Devices heading, before connected nodes, with the `Needs review` badge and `My fossil node` approval button.
- The named-volume coordinator database was preserved, and the pending node remained present after deployment.

## Recovery

Revert the UI and test changes, then rebuild the same test coordinator service. Compose data and credentials are held outside this source change and are unaffected.
