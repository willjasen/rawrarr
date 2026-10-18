# Show media and total metadata size

## Objective and initial state

Replace the Movies page's encrypted artwork count with metadata storage usage, and show the same figures on TV. The metadata service already exposed the encrypted metadata directory's total byte count; it did not separate the media subtree.

## Changes

- Extended `GET /api/metadata/storage` with `moviesBytes`, counting files under `metadata/movies` while retaining `bytes` for the full metadata directory.
- Replaced the Movies artwork summary with media metadata and all-metadata sizes; added the same summary to TV. Both pages read the storage endpoint.
- Sizes count regular files and display in binary units. The media subtotal currently covers the `movies/` directory, which is the repository's current media metadata namespace.
- Routed size reads through the authenticated coordinator-to-metadata relay. The direct metadata API did not expose this endpoint.
- Fixed Syncthing API key loading to fall back to the configured secret file when the corresponding Compose environment variable is empty. This was needed because rebuilding metadata with the current Compose environment otherwise left it unhealthy.

## Verification

- `git diff --check` passed for the edited Rust and browser source files.
- Rebuilt Roarlay and metadata with `docker-compose.yml`, `docker-compose.coordinator.yml`, and `docker-compose.tailscale-macbook.yml` in the existing `rawrarr-local` project. The first attempt used only the base file and failed on the sidecar-owned 7298 port; using the deployment's three-file Compose configuration succeeded.
- Both `rawrarr-local-roarlay-1` and `rawrarr-local-metadata-1` report healthy. The Tailscale Serve read-back maps HTTPS 443 `/` to `http://127.0.0.1:8080`, and the HTTPS health endpoint succeeds.
- Refreshed Chrome's existing Movies tab. It displays **1.1 GB** for both the current media subtree and full metadata total, with all 1,941 movie entries loaded. Both values match because current metadata files are all under `movies/`.

## Final state

Changes are deployed and visible in the existing Chrome Movies tab. Source changes are present in `rust-arr/src/main.rs`, `rust-arr/src/coordinator.rs`, `rust-arr/src/coordinator_workspace.html`, `rust-arr/src/coordinator-workspace.mjs`, and `rust-arr/src/syncthing.rs`. The worktree contains substantial pre-existing edits in these files, so the combined diffs require ownership review before staging or committing. Focused Rust/browser tests and a commit were not run/made.
