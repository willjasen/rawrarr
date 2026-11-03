# Build Rawrarr containers for the host architecture

Implement host architecture selection for source-built Rawrarr containers. GitHub issue creation and lookup were unavailable during this task; proposed issue: “Deploy containers for the host processor architecture”. Acceptance criteria: builds select a compatible AMD64 or ARM image for the target host and fail clearly for unsupported targets.

## Changes

- Use BuildKit's `TARGETARCH` in the Rust Docker build to compile `x86_64-unknown-linux-gnu` for AMD64 and `aarch64-unknown-linux-gnu` for ARM64.
- Reject other target architectures and verify the builder architecture marker matches the runtime stage target.
- Document the native local Compose build behavior and add a Dockerfile regression assertion.

## Verification

- `node --test scripts/tests/test_rawrarr_docker_healthchecks.mjs` passed all 3 tests.
- `git diff --check` passed.
- Docker build and Compose validation were attempted but could not connect to the OrbStack Docker socket (`permission denied`); cross-architecture image builds remain unverified in this environment.
