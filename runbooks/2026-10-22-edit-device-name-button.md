# Add an Edit button to coordinator device names

## Objective and initial state

Add an explicit Edit button beside each device name on the coordinator Devices page, then allow the owner to change and save the name through the existing browser-side age-encrypted config flow. The current field was always editable and saved on blur; the coordinator is deployed on the internal Mac mini.

Related context: [age-encrypted device names](2026-10-18-age-encrypted-device-names.md) and [internal mini coordinator deployment](2026-10-22-deploy-internal-mini-coordinator.md).

## Work performed

- Added a name group with an adjacent Edit button. The name field is read-only until Edit is activated; Enter or blur saves through the existing encrypted config flow, and Escape restores the saved value.
- Added a focused source assertion covering the new control and activation behavior.

## Verification and remaining work

- The focused new assertion passed (1/1). The full coordinator workspace suite has 13 passes and eight failures in older assertions spanning unrelated route labels, settings, profile, and node API behavior.
- `node --check rust-arr/src/coordinator-workspace.mjs` and `git diff --check` passed.
- Deployment remains pending because the coordinator source and stylesheet contain concurrent uncommitted device-delete and topology changes. Rebuilding from this mixed checkout would include work outside this task; do not deploy until those changes are isolated or integrated deliberately. Authenticated live editing remains unverified.
