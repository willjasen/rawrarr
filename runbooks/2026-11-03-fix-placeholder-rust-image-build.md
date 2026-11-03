# Fix placeholder Rust binary in container builds

Issue: [#434](https://github.com/willjasen/rawrarr/issues/434)

## Objective and initial state

Fix the manifest-only Cargo dependency layer so images always compile the real Rawrarr application after source is copied. GitHub Actions reported successful AMD64/ARM64 image builds for revision `381b775c9ec2b212b74c5e4d373e244f68e1be95`, but a subsequent host build from `4b2b0e20d99d2b48481bab078b0d5e16a50191b6` contained a 439,552-byte placeholder binary on ARM64. The coordinator auto-updater encountered the same failure on AMD64 and restored its prior image automatically.

## Change

After copying `src` and `migrations`, advance their mtimes before the second Cargo invocation. The dependency build creates a dummy `main.rs` newer than archived source files; refreshing copied source timestamps makes Cargo rebuild the actual application while preserving the manifest-keyed dependency cache layer. Add an ordering assertion to the focused Dockerfile regression test.

This changes only build behavior. It does not affect persistent data, schemas, or protocols, so no migration is needed. Existing deployments adopt the fix by building a new image from the merged source revision.

## Verification

- `node --test scripts/tests/test_rawrarr_docker_healthchecks.mjs` — 6 tests passed.
- Native AMD64 Docker build using the patched local tree — succeeded. Final image `rawrarr:codex-placeholder-fix-test` is `linux/amd64` and contains a 24,079,224-byte `rust-arr` executable.
- Executing that binary without `RAWARR_ROLE` returned the application's expected `Set RAWARR_ROLE explicitly` error (exit 1), confirming the executable is the application rather than the no-op placeholder.
- `git diff --check` — passed.
- ARM64 corrected build and published multi-architecture deployment remain pending.

## Recovery and final state

No service was deployed from the unpublished test image. The internal coordinator and Mac mini Fossil remain on their prior verified images. The do3 coordinator auto-update timer is stopped until a corrected image is published; restart it only after that image has passed startup/health verification. MacBook remains untouched until its SSH host identity can be verified.

Implementation is on branch `codex/fix-cargo-placeholder-image`, based on published `main` revision `4b2b0e20d99d2b48481bab078b0d5e16a50191b6`.
