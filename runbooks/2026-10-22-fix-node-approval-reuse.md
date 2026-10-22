# Fix node approval when a token is already enrolled

## Objective and initial state

Resolve the OrbStack coordinator's pending approval that returned no visible result. The pending request used a bearer token already assigned to the older approved `fossil` identity. A unique token index made approval fail, and the registration endpoint rejected the node's camelCase `displayName` field.

## Changes

- Accept camelCase registration fields while preserving strict unknown-field rejection and compatibility with clients that omit the optional name.
- When an approval request reuses an existing token for the same role and owner, update the existing approved device's display name, mark the request approved, and avoid inserting a duplicate credential. Requests whose token belongs to another owner or role remain conflicts.
- Extend the focused approval/registration source checks.

## Verification

- `node --test scripts/tests/test_rawrarr_node_approval_ui.mjs`: all six focused checks passed.
- `git diff --check` passed, and the Docker release build compiled the updated coordinator successfully.
- Recreated only `rawrarr-orbstack-test-roarlay-1`. The test coordinator HTTPS `/health` route returned HTTP 200, and Tailscale Serve still proxies HTTPS 443 to `http://127.0.0.1:8080`.
- Confirmed the pending request is still present as `MacBook Pro test1`. Approval remains an owner action; it was not clicked or simulated.

## Recovery

Revert the task-owned coordinator and focused-check changes, then rebuild the isolated test coordinator. No credential or node data needs to be deleted.
