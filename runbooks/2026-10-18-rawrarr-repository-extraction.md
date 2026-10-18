# Extract Rawrarr into its own repository

## Objective and initial state

Move the Rawrarr application out of the mixed `arr-services` Git repository into the sibling `Codebox/rawrarr` repository while keeping pre-Rawrarr Arr-services work out of the new repository. The source working tree was clean; the destination directory existed and was empty. Rawrarr code lived under `rust-arr/`, with its Compose files, documentation, tests, and runbooks mixed at the source root. The current OrbStack deployment and its local shared data are separate runtime state and must not be copied into Git.

## Actions

- Preserve history for the `rust-arr/` application only; add Rawrarr deployment, documentation, tests, and selected Rawrarr runbooks as Rawrarr-only project content.
- Exclude service-management scripts, media-library files, Arr service runbooks, runtime `/shared` data, local databases, environment files, and credentials.
- Remove the transferred Rawrarr source and documentation files from the `arr-services` working tree while preserving its other files and the historical Git commits.
- Keep the existing Rawrarr Docker deployment and shared-folder data intact during this repository relocation.

## Verification and remaining work

Verify each Git repository has the intended file set, run `git diff --check`, run Rawrarr-specific tests from the new root, and confirm the existing OrbStack service remains up. Neither repository has an upstream configured; do not create one during this extraction.
