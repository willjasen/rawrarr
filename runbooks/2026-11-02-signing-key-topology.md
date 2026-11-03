# Signing key topology and device sign-off

Issue: https://github.com/willjasen/rawrarr/issues/313

## Change

Fossils now presents a signing-key hierarchy alongside the existing Syncthing connectivity topology. The signed-in user is shown as the instance authority; enrolled devices are shown as root-authorized or legacy without a root-signed key. The displayed root fingerprint comes from the coordinator's pinned root record. Device approval states plainly that the user's root signature authorizes the exact device key and role, after the supplied age identity decrypts a backup whose fingerprint matches the independently saved value.

The passkey authenticates the account session. It does not authorize device membership; the instance root signature does. The topology is informational and does not grant or modify trust.

## Compatibility and rollout

No database schema, persisted data, shared files, or node/coordinator protocol changes are introduced. The workspace node response adds an optional root fingerprint field; old UI assets ignore it and new UI renders without it if the root has not yet been initialized. Legacy nodes remain available and are identified as not root-verified. Existing deployments adopt this UI/backend response change through the normal coordinator application image update; subscriber nodes do not require an update.

## Verification

`node --test scripts/tests/test_rawrarr_coordinator_workspace.mjs` passes (34 tests). The new browser geometry test is `scripts/tests/test_rawrarr_signing_topology_browser.mjs`; it could not run in this host checkout because `/usr/local/lib/node_modules/playwright-core/index.mjs` is absent. It must be run in the isolated Playwright test image to complete desktop (1280×800) and mobile (375×812) geometry verification. Rust compilation was not run because `cargo` is unavailable in this shell.
