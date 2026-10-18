# Refresh Rawrarr project documentation

## Objective

Update the project README, architecture guide, and design guide with the implemented product and setup flow, while keeping planned multi-node/service features clearly separated. Keep the user guide and technical status consistent with the current setup experience.

## Initial state

The root README was brief; [device architecture](../docs/device-architecture.md) mixed the current single-node prototype with target multi-node roles; and the user guide still described the old copy/checkbox order. [Product design](../docs/DESIGN.md) included the general flow, but not the exact acknowledgement label or the automatic generated-ID continuation.

## Changes

- Expanded the root README with current capabilities, the local Compose scope, and the reusable Docker test-base workflow.
- Added a current-prototype boundary to the device architecture guide, describing local PostgreSQL/passkeys, the browser-only age identity flow, encrypted shared config, SSE route, and health endpoint, separate from planned service/shard/storage deployment roles.
- Refined the design and user guides to describe the actual generated-ID modal sequence, copy acknowledgement, random character check, auto-navigation, and separate pasted-identity unlock action.
- Updated the technical status summary and created an index entry for this runbook.

## Verification

- Local Markdown links in the seven edited documentation files resolve.
- `git diff --check` passed. No code or application runtime changes were in scope.

## Final state

The README now describes the working local prototype and test workflow. The architecture guide separates the current single-node application from planned subscriber roles and hosted services. The design and user guides agree on the generated and pasted identity flows; remote proxy/SSE behavior is explicitly unverified.
