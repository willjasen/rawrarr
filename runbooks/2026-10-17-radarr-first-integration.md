# Radarr first integration

## Objective and initial state

Add a read-only first Radarr integration to the Rust service while keeping future Arr integrations possible. The committed skeleton exposes only `/health`; the repository is clean on `main`.

## Actions

Pending.

## Verification

Pending.

## Final state and remaining work

Pending.

## Actions

- Added optional Radarr configuration through `RADARR_URL` and `RADARR_API_KEY` without embedding credentials in Compose or source.
- Added `/api/radarr/status`, which performs a read-only request to Radarr's system status endpoint, exposes only app name/version/base path, returns generic upstream errors, and enforces a short client timeout.
- Kept health independent from Radarr availability. Added `Cargo.lock` and made the container build locked; configured Compose environment passthrough for local deployment.
- Used the current `reqwest` 0.13 line and verified crate versions against docs.rs on 2026-10-17: [reqwest](https://docs.rs/reqwest/latest/reqwest/) and [axum](https://docs.rs/axum/latest/axum/).

## Verification

- `cargo metadata --no-deps` passed and `docker compose config` passed.
- `git diff --check` passed.
- `cargo generate-lockfile` resolved 94 packages with Rust 1.85 compatibility.
- `cargo check --locked` and `cargo fmt --check` could not run because the installed `rustc` aborts while loading LLVM due to a missing Z3 dylib.
- Docker image build/runtime and a live Radarr connection were not verified: Docker daemon is unavailable. No production Radarr calls or writes were made.

## Final state and remaining work

The first Radarr integration is read-only and ready for build/runtime verification once the local toolchain and Docker daemon are healthy. Next useful increment: add sanitized local mock coverage and then implement movie inventory reads before any write operations. Future Sonarr, Lidarr, Readarr, and Bazarr clients remain out of scope for this milestone.
