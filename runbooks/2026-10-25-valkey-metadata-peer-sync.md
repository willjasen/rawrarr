# Use Valkey for fossil cache and plan peer metadata sync

## Objective and tracking

Use Valkey for the Rawrarr MacBook metadata-node artwork cache and record the longer-term goal of peer metadata exchange with authenticated RPC and Merkle-tree reconciliation. GitHub issue [#100](https://github.com/willjasen/rawrarr/issues/100) tracks implementation and acceptance criteria.

## Initial state

- The repository's MacBook and standalone metadata Compose files used `redis:7-alpine`; the cache was disabled by default.
- Before deployment, the MacBook runtime was Compose project `rawrarr-node-internal` from `/Users/willjasen/rawrarr-node-internal/compose.yml`. It had healthy fossil and Tailscale containers, no Redis/Valkey service, and no `RAWARR_ARTWORK_CACHE_ENABLED` variable in fossil's container environment.
- The fossil deployment lock was absent before this task claimed it.
- Related background: [initial artwork cache implementation](2026-10-18-implement-metadata-redis-cache.md), [deployment state](2026-10-25-deploy-tested-main-internal-macs.md), and [shared cluster plan](../docs/shared-cluster-sharing-plan.md).

## Changes

- Replaced the Redis server image with `valkey/valkey:8-alpine` and the server/health-check commands with `valkey-server` and `valkey-cli` in both metadata Compose variants. The application continues to use the Redis-compatible protocol URL.
- Preserved cache limits, loopback/private networking, disabled persistence, and opt-in default-off behavior.
- Added Valkey Compose regression checks to the existing node-network-isolation test.
- Added a planned architecture section for authenticated peer RPC and deterministic Merkle reconciliation. It explicitly states that neither RPC replication nor Merkle reconciliation is implemented.

## Verification

- `node --test scripts/tests/test_rawrarr_node_network_isolation.mjs` passed (5 tests).
- `docker compose -f docker-compose.metadata.yml config --quiet` passed with validation-only required variable paths and node ID.
- `docker compose -f docker-compose.yml -f docker-compose.tailscale-macbook.yml config --quiet` passed with validation-only secret-file paths and identity fingerprint. No secret file contents were read.
- Started isolated Docker network `rawrarr-valkey-verification` with `valkey/valkey:8-alpine` and a separate Rust test container from `rawrarr-testing-base:local`. `cargo test ... valkey_stores_and_returns_encrypted_artwork_when_configured -- --nocapture` passed against that Valkey service: PING succeeded and the exact encrypted fixture bytes were present under a versioned artwork key.
- `git diff --check` passed.
- PR [#102](https://github.com/willjasen/rawrarr/pull/102) passed its required GitHub Actions check and merged to `main` as `f7925d26652d683dc784c90ded2cadb3f6885a17`.
- Built `rawrarr:local` on the MacBook from a `git archive` of that exact published `main` commit. Image ID: `sha256:2d5b033cf2d4e0071cd6ab75c48a0ee6ea422d9d3eb35e8421afe84c608695cc`.
- Saved the host Compose backup at `~/Library/Application Support/Rawrarr/backups/pre-valkey-2026-10-25/compose.yml` with owner-only permissions. Added a Valkey 8 service sharing the existing Tailscale namespace, loopback-only bind, disabled persistence, 2 GiB maxmemory, 2.5 GiB container cap, and health check. Enabled the encrypted artwork cache on this internal fossil node.
- Atomically claimed `~/Library/Application Support/Rawrarr/.container-rebuild-fossil.lock` before deployment and removed it after verification. Recreated only `artwork-cache` and `fossil`; the Tailscale container ID and zero-restart state stayed unchanged. Fossil kept its `/data`, `/shared`, `/ui`, and token-secret mounts.
- Valkey returned `PONG`; `bind` is `127.0.0.1`, `appendonly` is `no`, and `maxmemory` is 2 GiB. Fossil reports `artworkCacheConfigured=true` and `artworkCacheReachable=true`; it is healthy with zero restarts. Local health and the MacBook tailnet HTTPS `/health` both returned HTTP 200. Tailscale Serve still maps `/` to `http://127.0.0.1:8080` and `/direct-api` to `http://127.0.0.1:8082`.
- Removed the disposable local Valkey verification containers and network after the integration test passed.

## Deployment and remaining work

## Final state and rollback

Internal MacBook `fossil` now uses Valkey for its encrypted artwork cache. The cache is enabled only on that node; roarlay and the coordinator were not touched. To disable/revert the Valkey deployment, restore the protected Compose backup and recreate only fossil with `--no-deps`; this removes the cache enable/URL settings while retaining all volumes. Remove the Valkey container only after fossil no longer depends on it. Do not delete `/shared` artwork or node volumes.

The Valkey service is only a node-local artwork cache. Peer metadata distribution via RPC and Merkle trees remains design work requiring a protocol specification, security review, two-node prototype, conflict/tombstone policy, and adversarial/offline tests.
