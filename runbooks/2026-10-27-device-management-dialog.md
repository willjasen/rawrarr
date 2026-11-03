# Device management dialog

Replace the standalone Fossils card Delete device action with Edit. The dialog allows an owner to rename a node in the existing browser-side age-encrypted config or delete it using the coordinator's existing revocation endpoint. Tracked by [issue #245](https://github.com/willjasen/rawrarr/issues/245).

## Acceptance criteria

- Each enrolled Fossils card has an Edit action in place of the direct delete action.
- Edit opens a dialog populated with the current display name.
- Saving a valid name uses the existing age-encrypted config flow and refreshes the card.
- Deletion is available from the dialog, retains the existing confirmation and coordinator revocation behavior, and explains that node-local data is not deleted.
- Cancel leaves the device unchanged.

## Compatibility and rollout

This changes only coordinator UI behavior. It does not change persisted formats, APIs, or node/coordinator protocols, so no migration is required. Existing deployments adopt it when the coordinator UI is updated.

## Verification

- `node --test scripts/tests/test_device_name_edit_button.mjs scripts/tests/test_rawrarr_device_names.mjs scripts/tests/test_rawrarr_node_approval_ui.mjs` — passed (13/13).
- `node --check rust-arr/src/coordinator-workspace.mjs` — passed.
- `git diff --check` — passed.

## Rollback

Restore the prior coordinator UI revision. No persistent state migration or rollback is required.
