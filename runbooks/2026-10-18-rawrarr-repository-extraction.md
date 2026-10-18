# Extract Rawrarr into its own repository

## Objective and initial state

Move the Rawrarr application out of the mixed `arr-services` Git repository into the sibling `Codebox/rawrarr` repository while keeping pre-Rawrarr Arr-services work out of the new repository. The source working tree was clean; the destination directory existed and was empty. Rawrarr code lived under `rust-arr/`, with its Compose files, documentation, tests, and runbooks mixed at the source root. The current OrbStack deployment and its local shared data are separate runtime state and must not be copied into Git.

## Actions

- Preserve history for the `rust-arr/` application only; add Rawrarr deployment, documentation, tests, and selected Rawrarr runbooks as Rawrarr-only project content.
- Exclude service-management scripts, media-library files, Arr service runbooks, runtime `/shared` data, local databases, environment files, and credentials.
- Remove the transferred Rawrarr source and documentation files from the `arr-services` working tree while preserving its other files and the historical Git commits.
- Keep the existing Rawrarr Docker deployment and shared-folder data intact during this repository relocation.

## Verification and remaining work

- The new repository history starts with the original Rust app skeleton and retains subsequent `rust-arr/` commits; it does not include earlier Arr-services-only commits.
- `docker compose config --quiet` passed for the extracted Compose files using the local password-file path (the path was not copied into Git).
- The two browser-side Node test files passed: 3 tests total.
- Host `cargo test` could not start because the installed Rust toolchain's LLVM library has a missing `libz3.5.1.dylib` dependency.
- OrbStack Compose configuration validation passed, and the Docker image built successfully from the new repository root (cached Rust release build).
- The existing OrbStack Rawrarr container remained running at `127.0.0.1:8084`, and `/health` returned `ok`. Its named volumes and old host shared-data directory were not copied or modified.
- Neither repository has an upstream configured; no remote was created.
