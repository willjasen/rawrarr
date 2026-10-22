# Select subscriber tailnet provider

## Objective and initial state

Add an internal-use option for subscriber nodes to join the owner's Tailscale tailnet while keeping public-service Headscale as a selectable path. The subscriber Compose generator previously had no overlay network sidecar. The shared checkout contains unrelated concurrent edits; this task limits changes to onboarding, Compose generation, tests, and this documentation.

## Changes

- Added a required onboarding choice: internal Tailscale or public Headscale.
- Added an auth-key input before Compose generation. It is held in browser memory, excluded from the preview/download, and cleared after downloading; the user saves it separately as the local Compose secret file.
- Generated a Tailscale sidecar with persistent Docker state and an auth key read from the node-local `tailscale-authkey` Compose secret file. The key is not embedded in generated YAML.
- Headscale mode requires a user-provided HTTPS control-server origin and adds its login-server flag. Internal mode uses the normal Tailscale control plane.
- Attached Rawrarr app traffic to the sidecar's userspace network namespace. This provides transport only; it does not grant Rawrarr approval or set up `/shared` Syncthing replication.

## Verification

`node --test scripts/tests/test_rawrarr_subscriber_compose.mjs` passed both tests. The browser bundle built successfully, and Docker Compose parsed both generated network variants. The browser test was attempted but cannot run in the host environment because its configured Playwright module is missing. Live tailnet enrollment remains unverified.

## Remaining work and limits

This is onboarding configuration only. Existing nodes are not migrated, the coordinator must be reachable from the selected tailnet, and owners must create a one-time auth key for the chosen control plane. No live tailnet enrollment or end-to-end node connection was performed.
