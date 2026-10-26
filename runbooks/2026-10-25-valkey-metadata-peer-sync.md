# Use Valkey for fossil cache and plan peer metadata sync

## Objective and tracking

Use Valkey for the Rawrarr MacBook metadata-node artwork cache and record the longer-term goal of peer metadata exchange with authenticated RPC and Merkle-tree reconciliation. GitHub issue [#100](https://github.com/willjasen/rawrarr/issues/100) tracks implementation and acceptance criteria.

## Initial state

- The repository's MacBook and standalone metadata Compose files used `redis:7-alpine`; the cache was disabled by default.
- The current MacBook runtime is Compose project `rawrarr-node-internal` from `/Users/willjasen/rawrarr-node-internal/compose.yml`. It had healthy fossil and Tailscale containers, no Redis/Valkey service, and no `RAWARR_ARTWORK_CACHE_ENABLED` variable in fossil's container environment.
- The fossil deployment lock was absent. This task did not modify the host-owned Compose file or deploy/restart the live node.
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
- `git diff --check` passed.

## Deployment and remaining work

No live deployment was performed. The MacBook host's active Compose file is outside this repository, and this branch has not been published to GitHub `main`. After code review/merge, update the host-owned Compose source to the published Valkey version, explicitly choose whether to enable the artwork cache, acquire the fossil deployment lock, and verify Valkey `PONG`, fossil health, cache reachability when enabled, and existing Tailscale Serve routes. Do not change roarlay or the coordinator.

The Valkey service is only a node-local artwork cache. Peer metadata distribution via RPC and Merkle trees remains design work requiring a protocol specification, security review, two-node prototype, conflict/tombstone policy, and adversarial/offline tests.
