# Implement metadata-node Redis artwork cache

## Objective and initial state

Implement the optional Redis artwork cache for metadata nodes. `/shared/metadata` remains authoritative, Redis stores only encrypted artwork ciphertext, and cache failure falls back to shared files. The current local tree measured about 1.13 GiB of artwork; the feature remains disabled by default.

The checkout already contained extensive unrelated user changes. Changes in shared files were limited to cache-specific hunks; no existing changes were reverted. Deployment target is the MacBook Pro `rawrarr-local` project only; no Mac mini deployment was attempted.

## Changes

- Added the `artwork-cache` Redis service to local and standalone metadata Compose deployments. It has no published port or persistence, binds only to loopback in the shared MacBook Tailscale namespace, uses 2 GiB maxmemory with LRU eviction, and has a 2.5 GiB container memory limit.
- Added opt-in `RAWARR_ARTWORK_CACHE_ENABLED`, Redis URL, and TTL configuration. The toggle defaults off.
- Added a Rust cache adapter for encrypted ciphertext and connected movie and TV poster reads to it. The reader hashes the authoritative file and uses a path+content-digest key, so Syncthing replacements use a new key. Redis errors fall back to the file path.
- Added focused Rust tests for disabled mode and unreachable-Redis fallback.

## Verification

- `docker compose -f docker-compose.metadata.yml config --quiet` passed with a validation-only `RAWARR_NODE_ID`.
- `docker compose -f docker-compose.yml config --quiet` passed with the existing protected password-file path; the secret value was not read.
- `git diff --check` passed for the cache implementation and Compose files.
- Rust compilation was attempted via the project Docker build because local rustc aborts while loading the unavailable `libz3.5.1.dylib`. `docker compose ... build metadata` passed using Rust 1.85 in Docker. `cargo test --locked artwork_cache` passed (2 tests). `node --test scripts/tests/test_rawrarr_device_routes.mjs` passed (3 tests). `test_rawrarr_direct_route.mjs` has two failures in assertions against current pre-existing direct-route/UI source; the cache route test itself passed. `cargo fmt --check` was unavailable because the Rust Docker image does not include the rustfmt component.

## Limitations and remaining work

- The implementation hashes and reads the source file on every lookup to ensure external Syncthing replacements cannot make stale entries valid. This primarily avoids repeating Redis misses/decryption and does not eliminate the current disk read; benchmark before treating it as a latency improvement.
- Cache observability/status UI and a cache-clear operation are not included in this initial implementation.
- Redis and metadata were deployed to MacBook Pro `rawrarr-local` with `RAWARR_ARTWORK_CACHE_ENABLED=true`. Redis health returned `PONG`, metadata direct health reported cache configured and reachable, and local `:7297/health` returned HTTP 200. Redis reports `bind 127.0.0.1`, has no published port binding, uses 2 GiB maxmemory, and has a 2.5 GiB container limit. The existing Tailscale Serve handlers were read back and remained `/` to roarlay `127.0.0.1:8080` and `/direct-api` to metadata `127.0.0.1:8082`. The Mac mini was not changed.
