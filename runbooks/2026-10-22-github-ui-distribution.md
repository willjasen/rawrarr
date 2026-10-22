# Separate Rawrarr UI updates from node images

## Objective and plan

Allow UI changes merged to GitHub to reach coordinator and subscriber nodes without rebuilding their application containers for every UI release. Preserve same-origin API and passkey behavior, and keep node-local configuration/data in existing mounts.

### Planned architecture

1. Move UI templates, browser modules, and static assets into a versioned UI bundle, separate from Rust API code.
2. Publish an immutable bundle and checksum from GitHub Actions when UI source changes on the default branch.
3. Add a small updater on each node that periodically checks the published version, verifies the checksum, downloads to a staging directory, validates paths, and atomically switches the active bundle. Keep the previous bundle for rollback.
4. Change the Rust service to serve the active bundle from a dedicated mount, retaining embedded UI as first-boot and network-failure fallback. APIs, auth, and cookies remain same-origin.
5. Add the updater mount/service to coordinator and subscriber Compose generation. This is a one-time deployment change; later UI-only commits should update without rebuilding application containers.
6. Verify both coordinator and subscriber route paths, auth flows, and rollback behavior before claiming deployment complete.

## Initial state

- `main.rs`, `auth.rs`, and `coordinator.rs` embed HTML and browser assets with `include_str!`/`include_bytes!`.
- Coordinator and subscriber UI share the Rust image and same-origin API/auth behavior.
- Existing GitHub Actions only runs script tests; no UI bundle publishing or node update mechanism exists.
- Work began in an isolated worktree because the shared checkout contains unrelated changes.

## Implementation status

Planning recorded. No application code or live deployment changed yet. The architecture requires a coordinated first-time service and Compose change; publishing an artifact alone would not update running nodes. Before implementation, inventory all UI templates/assets and all supported Compose generation paths, then build and validate the updater and dynamic asset loader as one cohesive change.

## Verification and recovery

Not yet performed. The implementation must test checksum rejection, interrupted downloads, unsafe archive paths, rollback to the previous bundle, embedded fallback, and UI-only publication without application image rebuild. Deployment rollback is to disable the updater and serve the retained prior bundle or embedded UI.
