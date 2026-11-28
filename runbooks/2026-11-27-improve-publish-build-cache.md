# Improve published image build cache

- Objective: improve ARM64 and AMD64 Rust compilation reuse in the published container workflow. GitHub issue: [#538](https://github.com/willjasen/rawrarr/issues/538).
- Initial evidence: run 46098056988 imported the per-architecture BuildKit cache, but rebuilt Cargo dependencies on both architectures (roughly 99 seconds ARM64 and 143 seconds AMD64). The previous successful image run was only about 15 minutes earlier. Workflow already used `type=gha`, `mode=max`, and separate architecture scopes.
- Change: add `sccache` to the native-architecture image jobs, using its GitHub Actions cache backend. Pass the runtime cache URL and token into each Cargo build only through BuildKit secret mounts; no cache credentials are build arguments or image layers. This supplements the existing BuildKit layer cache and does not change runtime behavior or require a data migration.
- Safeguard: preserve the existing real post-source build and timestamp refresh, preventing the placeholder binary from entering the final image. Cache credentials are mounted only during Cargo commands. Add focused workflow and Dockerfile assertions.
- Verification: pending focused tests and workflow review.
- Rollback: revert the sccache workflow and Docker build-argument changes; compiler cache data is build-only and does not affect deployed state.
- Remaining: observe a subsequent GitHub Actions image build to measure actual hit rate and duration.
