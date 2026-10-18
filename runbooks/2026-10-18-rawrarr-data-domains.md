# Establish Rawrarr data domains

## Objective and initial state

Define stable paths for media, encrypted metadata, Syncthing-shared app state, and node-local state. The current deployment already mounted the Syncthing folder at `/shared`; library sidecars and posters were physically stored under `/shared/metadata`, but `/metadata` was not yet a named domain.

## Actions

- Established `/media` for storage-node media, `/metadata` for shared encrypted metadata, `/shared` for the Syncthing root, and `/data` plus the PostgreSQL volume for node-local state.
- Made `/metadata` a Docker image alias of `/shared/metadata`, and configured both the web container and storage-only scanner to use that path. Native execution defaults to `$RAWARR_SHARED_DATA_DIR/metadata`.
- Added a data-domain registry documenting node roles, persistence, encryption, sync boundaries, and current limitations. User/passkey synchronization remains planned, not implemented.
- Updated the device architecture and Rust deployment documentation to link to the registry.

## Verification

- Rust 1.85.1 `cargo fmt` and locked `cargo test` passed (6 tests); the new domain test covers the default shared subtree and `/metadata` alias setting.
- Rebuilt and restarted only the Rawrarr app in OrbStack. An in-container check confirmed `/metadata` resolves to `/shared/metadata`; `/health` returned `ok` and PostgreSQL stayed healthy.
- `git diff --check` passed. No media scan was run.

## Final state and remaining work

The domain paths are established in the image and documentation. Remote scan coordination and portable synchronized passkey records remain future work.
