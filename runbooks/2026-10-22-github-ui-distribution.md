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

- Added a strict allowlist and packaging script for the current HTML, JS modules, and image assets. GitHub Actions publishes a private release for commits to `main`.
- Added a Rust UI loader that reads the per-node `/ui/current` tree for all existing UI routes and falls back to the embedded files. API routes and authentication remain on the same origin.
- Added an updater to the application image. When the private GitHub token secret is mounted, it checks the latest release every five minutes, downloads the bundle and checksum through the GitHub API, rejects path traversal and oversized content, and swaps the staged directory atomically. The previous bundle is retained; rollback uses `RAWARR_UI_DIR=/ui/previous` and `RAWARR_UI_UPDATES_ENABLED=false`.
- Added token and UI-volume wiring to coordinator, metadata, storage, local Compose files, and subscriber Compose generation. Each service uses a separate volume to avoid concurrent updater writes.
- Added focused updater/package tests and updated deployment guidance. No live service was rebuilt or deployed; every node still needs the one-time image/Compose update and a protected token file.

## Signature hardening consideration

The current SHA-256 sidecar detects accidental or in-transit corruption only; it does not prove publisher identity because the same GitHub release can replace both files. Before treating this as authenticated UI delivery, add a detached signature over the complete bundle and verify it on the node before installation. Browser verification needs a small stable verifier and public key pinned in the application image; the verifier must be available before any downloaded UI executes. Keep the signing private key in a protected release signing environment, never in node containers. A valid signature authenticates the publisher's code but does not limit what that browser code can read or transmit.

## Verification and recovery

Focused Python bundle/updater tests (3/3) and subscriber Compose generator tests (3/3) pass. The Rust release image built successfully in Docker, and the updater `--help` runs in the host Python environment. Host `cargo check` is unavailable because the installed LLVM cannot load its missing Z3 dylib. Compose parsing passed for the main coordinator, internal-mini coordinator, metadata, storage, and MacBook-storage files. GitHub Actions has not run yet, and no live node refresh has been verified. The current implementation uses checksums, not digital signatures; the signed browser verification is a follow-up security enhancement.
