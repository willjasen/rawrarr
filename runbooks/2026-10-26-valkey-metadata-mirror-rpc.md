# Replicate config.aged through Valkey and peer RPC/Merkle

## Objective and tracking

Replicate only encrypted `/shared/config.aged` through authenticated RPC/Merkle between two fossils, with a local Valkey ciphertext mirror. Keep app metadata and artwork in Syncthing. GitHub issue [#100](https://github.com/willjasen/rawrarr/issues/100) tracks the scope and acceptance criteria.

## Initial state

- Created the isolated branch `codex/valkey-metadata-rpc` from `origin/main` at `421dca34`, then fetched and rebased onto newly published `origin/main` at `e530450` before implementation. The newer base includes the group-encrypted config bootstrap fix.
- The MacBook `rawrarr-node-internal` fossil and its Valkey service are healthy. The existing Valkey service is the optional encrypted-artwork cache. `/shared/config.aged` exists there and is 2,060 bytes; its contents were not read.
- The Mac mini `rawrarr-internal` fossil is healthy, has no Valkey container, and has no `/shared/config.aged`. Its separate `rawrarr-internal-mini` roarlay, PostgreSQL, and Tailscale services must remain outside subscriber changes.
- Neither live fossil has `/data/radarr-config.json`. The MacBook `config.aged` begins with the age ciphertext marker; no file contents were read. Preserve this property: local files and Valkey contain ciphertext only, while decrypted settings remain in memory when needed.
- Neither shared root had a `.stignore` file. Native Syncthing processes are running on both Macs. Syncthing's ignore file is local to each folder and must be set independently.
- Both fossil containers have node-local signing-key files. Group-age identity files are currently absent on both; this requires compatibility review before asserting that the restored config can be decrypted by both live nodes.
- Current source stores only encrypted artwork in Valkey and has no peer RPC/Merkle config repair. The coordinator currently returns only the authenticated node's own signing certificate.
- The user narrowed the scope to `config.aged` only: RPC/Merkle transfers that ciphertext and its signed revisions; app text metadata and artwork remain Syncthing-managed. The two-node phase tests reconnect repair only, not automatic failover or RF=3.

## Actions

- Updated issue #100 with the narrowed config-only scope, local Valkey rebuild, authenticated bounded RPC, Merkle anti-entropy, two-node reconnect tests, and per-node Syncthing exclusion requirement.
- The user selected RPC/Merkle-only transfer for `config.aged`; do not configure an RPC/Syncthing dual-writer path.
- The user explicitly requires no plaintext config copy on node-local disk. Revision journals must contain only ciphertext, signed digests, counters, and opaque IDs.
- Fetched the newer `origin/main` at `e530450` and rebased this task branch; no other worktree or branch was changed.
- Keep all operational deployment work on published GitHub `main`; no live service or Syncthing ignore rule was changed during inspection.
- The user approved the protocol draft. Added a ciphertext-only Valkey cache keyed by pinned root fingerprint and ciphertext SHA-256, startup/periodic mirroring from the active file, local Valkey-to-file rehydration when the active file is missing, age ciphertext bounds checking, and deterministic revision/Merkle primitives.
- Updated metadata Compose variants to give the app a local Valkey URL; expanded the isolated two-node test Compose with independent `/shared` volumes and Valkey instances.
- Set `RAWARR_LOCAL_PLAINTEXT_CONFIG=false` for the MacBook metadata overlay and both isolated test fossils so node containers do not write a plaintext settings mirror.
- Added Rust regression coverage for ciphertext validation, Merkle stability, and deterministic conflict ordering.
- Added pinned-root certificate validation, one-use signed peer challenges, bounded fixed-object manifest/config routes on the direct listener, durable signed revision storage, Valkey revision indexing/restoration, and 30-second peer reconciliation. Missing config startup no longer creates an empty shared config when a valid tailnet peer allowlist is configured.
- Read the live, non-secret deployment identity fields: both fossils are metadata nodes, with IDs `device-d3ee53b1d561fb349955b199` (MacBook) and `device-86fc42158c1426b89d460f59` (Mac mini); each has the direct listener on port 8082.
- The Mac mini subscriber Compose project currently has no Valkey service. Deployment must add a dedicated Valkey container to that subscriber project and keep it separate from `rawrarr-internal-mini` coordinator services. Both nodes already have loopback-only direct listeners exposed through their `/direct-api` Tailscale Serve route.

## Verification

- Read the current shared-cluster plan, device-signing design, Valkey artwork-cache implementation, and prior Valkey/peer-sync runbooks.
- Read-only SSH/Docker inspection confirmed the MacBook fossil and Valkey are healthy; config ciphertext exists there. The Mac mini fossil is healthy, has no Valkey service, and lacks config ciphertext. No container was restarted and no config plaintext was read.
- Read-only checks confirmed `/data/radarr-config.json` is absent on both live nodes; the MacBook config file has the expected age ciphertext prefix. No plaintext configuration was read or printed.
- Confirmed both Macs run native Syncthing and neither `/shared` root has `.stignore`. No ignore file was written. The official Syncthing guidance says `.stignore` is local to each synchronized folder and is not propagated to other devices; add and verify the root-level `/config.aged` rule on both nodes only after both run the RPC-capable published release.
- Confirmed issue #100 is open and updated its title and acceptance criteria.
- `docker compose -f docker-compose.orbstack-test.yml config --quiet` passed after separating the test fossils' shared volumes and adding per-node Valkey services.
- `docker compose -f docker-compose.orbstack-test.yml config --quiet` and the standalone metadata Compose config passed. The MacBook Compose file is an overlay, so it must be validated alongside the base deployment file and runtime settings.
- `git diff --check` passed.
- Focused `cargo test --locked --offline config_repair` in the isolated Rust image passed the ciphertext, allowlist, atomic-write, Merkle, signed revision, replay, and peer-manifest exchange tests.
- `python3 scripts/tests/run_rawrarr_tests_in_docker.py` passed after RPC wiring: 110 Node tests, the browser suite, and 94 Rust tests. The runner skipped Python environment integrations.

## Status and remaining work

The approved protocol is in [metadata-peer-repair-protocol.md](../docs/metadata-peer-repair-protocol.md). RPC, signed journal, and periodic Merkle reconciliation are implemented locally; the complete isolated convergence gate and live rollout remain. Live Syncthing exclusions and production services have not been changed. Preserve `/shared/metadata`, artwork, the mini coordinator, and both fossils' node-local state.
