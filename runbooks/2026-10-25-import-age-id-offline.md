# Import the age ID while nodes are offline

## Objective

Allow the user to import and save their age private ID in this browser without an online fossil. Track [issue #82](https://github.com/willjasen/rawrarr/issues/82).

## Initial state

Settings posted the entered identity to the selected metadata node's direct `/settings/age` endpoint before storing it. With no online node or direct Tailscale route, the browser displayed the route-selection error and could not retain the identity.

## Actions

- Validate the private identity locally through the existing browser age implementation by deriving its public recipient; do not send the key to roarlay or require a node for this browser-storage action.
- Persist only after successful local validation; preserve existing localStorage key and clear behavior.
- Add browser regression coverage for no-node import and invalid identity rejection.

## Verification

- The isolated `rawrarr-test-regression` runner passed all Node tests (128), its Chromium browser test (1), and Rust unit tests (82). Python integration checks were skipped by the runner because they require separate fixtures.
- The first full run exposed an unrelated movie responsiveness timeout; a later run passed all suites.

## Recovery and remaining work

The user can clear the locally saved ID from Settings. Node-dependent library access and encrypted-config actions continue to require an available node.
