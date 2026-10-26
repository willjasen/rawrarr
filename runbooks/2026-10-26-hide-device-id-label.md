# Hide generated device IDs from device labels

Issue [#155](https://github.com/willjasen/rawrarr/issues/155)

## Objective

Remove opaque generated IDs from node names in the coordinator Devices topology while keeping stable IDs for routing, peer matching, and device management.

## Initial state

- Started from fetched `origin/main` in the dedicated branch `codex/hide-device-id-label`.
- The topology renderer used `node.nodeId` as the final visible-name fallback when neither an age-configured alias nor an enrollment display name was available.
- The Devices page already renders the configured/enrollment display name plus role in each card.

## Actions

- Changed the topology's last-resort label to the node role rather than its opaque ID.
- Updated existing source regression assertions that encoded the old fallback.

## Verification

- `node --test scripts/tests/test_rawrarr_node_approval_ui.mjs scripts/tests/test_device_name_edit_button.mjs` — 9 tests passed.
- `node --test scripts/tests/test_rawrarr_coordinator_workspace.mjs scripts/tests/test_rawrarr_node_approval_ui.mjs` — 35 passed and 1 failed. The failure is an existing stale assertion in `test_rawrarr_coordinator_workspace.mjs` expecting the Devices page's add-device link immediately after its heading; the current page layout places it elsewhere. The other tests passed, including the changed topology-label assertion.
- `git diff --check` — passed.

## Final state

The topology displays a configured alias or enrollment name when available, and uses the node role if the only available display name is the generated stable ID. The stable ID remains in the data model and internal management paths. Changes are local and uncommitted on `codex/hide-device-id-label` for review.

## Remaining work

- Review the focused UI change and decide whether to commit/publish it.
- The broader workspace test's stale add-device-link expectation is outside this issue.
