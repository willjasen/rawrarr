# Optional in-memory artwork cache plan

The Redis-compatible cache service is Valkey (`valkey/valkey:8-alpine`). The Rust client and URL scheme remain Redis-protocol compatible (`redis://`).

## Goal

Let an operator opt a Rawrarr node into a bounded Redis cache for shared encrypted artwork. Cache hits should reduce repeated shared-folder reads while preserving the current age-encryption boundary. The cache is disposable: `/shared/in/metadata` remains authoritative and every miss or Redis failure falls back to the existing file read and decrypt path.

This is a node-local performance feature, not synchronized app data. Redis must not be added to the coordinator or relay path and must never receive private age identities, plaintext metadata, media, credentials, or decrypted artwork. Cache only age-encrypted `.aged` artwork bytes. Text metadata is small and already has an unlocked-vault-local parsed cache, so it is out of scope for the first implementation.

## Sizing evidence

The current local metadata tree contains 4,349 files and 1,251,738,402 bytes (about 1.17 GiB): 2,172 artwork files use 1,213,605,022 bytes (about 1.13 GiB), while text records use about 39 MB. Artwork averages about 559 KiB across the collection, with a largest observed file of about 2.9 MB. These values are a point-in-time local inventory, not a promise about another node or future growth.

An opt-in 2 GiB Redis `maxmemory` budget is a reasonable initial ceiling for this observed collection, but maxmemory is not the whole container/process RSS. Redis object, allocator, client-buffer, and fragmentation overhead must be measured using representative keys. Use an explicit Docker memory limit above the Redis maxmemory value, with operational headroom. The design must support partial caching and eviction; do not promise that every object remains resident.

## Proposed behavior

- Add an opt-in per-node configuration, disabled by default. Local Compose and standalone metadata deployments can enable it; production operators must explicitly enable and size it for each node.
- Keep Redis on a private Compose network, with no published host port. Persist no cache data and disable snapshots/AOF for this disposable cache.
- Use a bounded `allkeys-lru` (or measured equivalent) eviction policy and a configurable maxmemory value whose initial documented default when enabled is 2 GiB. Validate configuration and fail closed to the existing file path if Redis is unavailable or misconfigured.
- Use a versioned, collision-resistant cache key derived from a hash of the opaque artwork path and the current file generation (inode, size, and nanosecond modification time). Syncthing atomic replacements then produce a new key, and cache hits can return ciphertext without rereading the file. Avoid putting media IDs, titles, filesystem paths, or age identifiers into Redis keys or logs.
- On read, stat the current file to identify its generation and look up encrypted bytes. On a miss, read ciphertext from `/shared` and populate Redis best-effort. Decrypt with the unlocked local vault as today. A cache hit still requires normal decryption before bytes are served.
- Ensure updates, replacement, deletion, and HMAC filename migration cannot return stale artwork. Prefer a digest/version check rather than relying only on a best-effort invalidation event; synchronized changes may arrive outside the Rawrarr process.
- Keep the cache per node, since each approved node has its own available RAM and local process. Do not route cache traffic through the coordinator.
- Add an operator-facing setting/status that clearly reports disabled/enabled state, configured budget, Redis reachability, current memory use, key count, hit/miss/eviction counts, and fallback errors. Never expose cached content or secret values.
- Do not preload the whole tree initially. Warm entries on demand. Add an explicit bounded warm-up only if measurements later show cold-start latency matters and the operator enables it.

## Implementation phases

1. **Baseline and interface:** measure artwork read/decrypt latency and current request patterns on the metadata node. Define config names, defaults, cache key format, metrics, and fail-open behavior. Keep the current endpoint behavior unchanged when disabled.
2. **Redis service wiring:** add an optional Redis service/profile to the local and metadata Compose variants, with no published port, no persistence, health check, memory controls, and an isolated private network. Update deployment documentation and node-local config guidance.
3. **Cache adapter:** implement a small Rust abstraction for get/set/delete-if-needed with bounded timeouts and payload limits. Redis failures must never fail artwork serving; they log redacted operational errors and use shared-file fallback. Store ciphertext only.
4. **Read-path integration:** connect the adapter to movie and TV artwork loading. Use a content digest/version to prevent stale cache hits after Syncthing updates, preserve vault unlock/decryption behavior, and make cache writes best-effort.
5. **Operations UI and observability:** add a per-node enable/disable control or documented config path, current memory and hit/miss/eviction status, and an explicit clear-cache action. Clearing Redis must not touch shared files. Ensure status can distinguish Redis health from actual cache effectiveness.
6. **Validation and rollout:** test disabled behavior, encrypted-byte-only storage, hit/miss behavior, changed/deleted/replaced files, stale-entry rejection, Redis-down fallback, full-cache eviction, and bounded-memory behavior. Benchmark warm versus cold artwork response latency and measure Redis `used_memory`/container RSS on a sanitized representative fixture and then the local deployment. Roll out one metadata node first and keep the feature opt-in until results support the budget.

## Acceptance criteria

- With caching disabled, the existing shared-file/decrypt path and responses are unchanged.
- With caching enabled, a repeat request can hit Redis, but the cached value is still encrypted and is decrypted only by the node's existing unlocked vault.
- Redis outage, timeout, eviction, restart, or empty state never prevents artwork from being served when the shared file and vault are available.
- A synchronized artwork replacement or deletion cannot cause stale artwork to be served; stale entries are rejected even if explicit invalidation was missed.
- Redis is reachable only by the owning Rawrarr node/container network, is not coordinator-accessible, and contains no plaintext content or credentials.
- Memory use stays within configured Redis maxmemory; process/container memory has documented headroom. Status reports hit rate, evictions, errors, and actual memory use.
- The feature is off by default, node-local, and does not change `/shared`, Syncthing, or age-encryption formats.

## Initial implementation status

The initial opt-in implementation is deployed on the MacBook Pro metadata node. Redis stores encrypted artwork ciphertext only, uses a 2 GiB LRU limit, and binds to loopback in the shared MacBook Tailscale namespace. The coordinator's direct health probe reports Redis configured/reachable state and measures the Redis ping as part of node route latency. Cache telemetry for hit rate, evictions, and current memory use, a user-facing enable/disable control, and a clear-cache action remain future work. Generic Compose settings remain disabled by default.

## Risks and open decisions

- Redis maxmemory does not strictly cap process RSS; allocator fragmentation and buffers require headroom and measurement.
- Generation keys rely on inode, size, and nanosecond modification time. Syncthing replacement writes normally change inode; test filesystems and sync methods to confirm generation changes. If same-inode in-place rewrites are supported, add a safe invalidation mechanism or digest validation before relying on warm hits.
- Redis adds another service and operational failure mode. If latency improvement is small, a local bounded in-process ciphertext cache may be simpler.
- A 2 GiB ceiling fits the measured collection's raw artwork total only with modest overhead; it is not a guarantee of a complete resident set. Growth, future artwork, and multiple simultaneous clients may require a larger budget or continued eviction.
- Redis authentication/TLS is unnecessary if network isolation is enforced within a single-node Compose deployment, but must be revisited if Redis becomes remotely reachable or shared across nodes.

## Related boundaries

- [Data domains](data-domains.md) defines `/shared`, `/metadata`, and node-local state.
- [Container deployment](container-deployment.md) describes the current coordinator, storage, and metadata roles.
- [Technical status](technical-status.md) distinguishes implemented cache behavior from planned features.
