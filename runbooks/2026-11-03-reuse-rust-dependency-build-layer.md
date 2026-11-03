# Reuse the Rust dependency build layer

Issue: [#188](https://github.com/willjasen/rawrarr/issues/188)

## Objective and initial state

Separate Rust dependency compilation from Rawrarr source compilation so source-only image builds can reuse the dependency layer through the existing AMD64 and ARM64 GitHub Actions BuildKit caches. The original `rust-arr/Dockerfile` copied `src` and `migrations` before its only Cargo build. The task began on a clean branch based on `origin/main` (`b3128b8`).

## Changes

- After copying `Cargo.toml` and `Cargo.lock`, create a temporary `main.rs` and empty migrations directory, then run the locked release build for the requested target architecture.
- Keep `/src/target` out of a BuildKit cache mount so compiled dependency artifacts are included in the intermediate builder layer and exported by `cache-to: type=gha,mode=max`. Keep Cargo registry and Git source cache mounts.
- Copy real source and SQLx migrations after that build, then run the locked release build again. A source or migration change invalidates this final build step while preserving the manifest-keyed dependency layer. Manifest or lockfile changes invalidate dependency compilation.
- Add a focused Dockerfile regression assertion and update the test index entry.

No existing deployment state, schema, or protocol changes; no migration is needed. Published container builds adopt the layer automatically using their existing architecture-specific GHA cache scopes.

## Verification

- `node --test scripts/tests/test_rawrarr_docker_healthchecks.mjs` — 6 tests passed.
- `git diff --check` — passed.
- Confirmed `.github/workflows/publish-container.yml` uses `cache-from`/`cache-to` with `scope=rawrarr-${{ matrix.arch }}` and `mode=max` for each architecture.

No full container image build or GitHub Actions run was performed. The test verifies Dockerfile ordering and cache-mount invariants; end-to-end BuildKit reuse will be observed in subsequent CI image builds.

## Final state

Implementation and focused coverage are ready for commit on `codex/issue-188-cargo-docker-cache`. No deployment performed.
