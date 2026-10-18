# Faster local development builds

## Objective

Shorten the time required to rebuild Rawrarr after source changes, while keeping `rawrarr-local` available until the user requests an update.

## Initial state

The Rust app Dockerfile copied all sources before `cargo build`. A source edit invalidated that build layer, and neither the Cargo registry nor `/src/target` was persisted between builds. Rebuilding the local image compiled and downloaded the full dependency graph. Existing unrelated changes in `docs/node-types.md`, `runbooks/README.md`, and `runbooks/2026-10-18-rawrarr-container-distribution.md` were preserved.

## Changes

- Added BuildKit cache mounts for Cargo's crate registry, Git database, and target directory in the Rust build stage.
- Documented the agreed workflow: implement and verify code changes first; rebuild the persistent local environment only when the user requests hands-on testing.

## Verification

Build verification passed on the OrbStack BuildKit builder. The initial cache-seed release build took about 4m08s for Cargo. A temporary source-only edit then rebuilt the Rust app in 14.22s (21.2s total); Cargo dependencies were reused, and the Debian runtime and age tool layers were cached. The temporary image and source copy were removed. `rawrarr-local` remains running and healthy on its existing image; it was intentionally not rebuilt for this workflow change.

## Recovery and remaining work

Remove the three cache mounts to restore the prior clean-build behavior. The caches contain downloaded dependencies and compiler artifacts only; they are not application state. The first attempt exposed that a cache-mounted `/src/target` is unavailable to a later image-copy step; the Dockerfile now copies the compiled executable to `/rust-arr` before the cache mount ends.
