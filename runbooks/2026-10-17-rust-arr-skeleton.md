# Rust Arr skeleton

## Objective and initial state

Start a new memory-safe Rust service skeleton in this repository, prepared for local development and Docker deployment. The repository was clean on `main`. Project background is in [the movie organization runbook](2026-10-16-organize-remaining-loose-movies.md).

## Actions

- Added `rust-arr`, a small HTTP service with `GET /health`, configurable listen address, Rust 2024 edition, and `unsafe_code = "forbid"`.
- Added a multi-stage Dockerfile that builds with Rust 1.85 and runs as non-root UID 10001, plus root-level Compose configuration.
- Documented local and Docker usage and the limits of a crate-local unsafe prohibition. No media mounts, database, or credentials are configured.

## Verification

- `git diff --check` passed.
- `docker compose config` passed and resolved the intended service and port mapping.
- Docker image build could not run because the Docker daemon is unavailable (`docker.sock` missing).
- Native Rust build could not be verified: the installed `rustc` fails to load LLVM because its expected Z3 dylib is missing.

## Final state and remaining work

The source and deployment skeleton are ready for review. Build and runtime behavior remain unverified until the local Rust toolchain and Docker daemon are available. No live Arr service or media library was touched.
