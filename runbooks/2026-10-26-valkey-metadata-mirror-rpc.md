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

## Published deployment

- PR [#159](https://github.com/willjasen/rawrarr/pull/159) was merged to GitHub `main` as `99e2e82d2e1953950a7820800018a37500618919`. The deploy classifier selected both coordinator and nodes relative to deployed revision `83d70be134a1a56e81a109ce62a75ad13632d17c`.
- Built the Mac mini image from the published `main` archive on the mini's native ARM64 OrbStack host. The resulting image is `linux/arm64`; the mini fossil and `roarlay` both run it without runtime emulation. MacBook fossil runs the `linux/amd64` published image on its `x86_64` host.
- Added a dedicated Valkey service to the Mac mini subscriber Compose project. It is isolated in the fossil Tailscale network namespace, listens on loopback, has a volatile 256 MiB cache, and is healthy. The MacBook's existing Valkey service remains in use.
- Configured both fossils with the same stable node-ID allowlist and HTTPS `.ts.net` direct API URLs. Both local plaintext-config flags are false. The RPC endpoints use the node `/direct-api` Tailscale Serve routes.
- Recreated mini fossil, MacBook fossil, and mini coordinator `roarlay` from the published image; preserved PostgreSQL and Tailscale services and state. Both fossils, both Valkey services, and `roarlay` are healthy. Valkey returns `PONG` on both fossils.
- The mini initially had no `/shared/config.aged`; after the RPC release it received the 2,060-byte age ciphertext. Both active files now have the same SHA-256 (`3a99ac9d4e7437e849c1a2969eb542e61fda6eb3589a68bf4f4453389d4360aa`) and mode `0600`. Each Valkey current-digest pointer matches its active file, and a SHA-1 computed inside Valkey confirms the cached bytes match the active ciphertext. The mini's durable journal contains the MacBook's signed `put` revision, confirming peer revision transfer.
- `/data/radarr-config.json` is absent on both nodes. No age identity was read or printed. Direct `/direct-api/health` routes on both `.ts.net` hosts return HTTP 200. The mini tailnet Serve read-back still maps `/` on port 443 to `http://127.0.0.1:8080`; the public health route responds as Roarlay.
- A local ARM64 cross-build was stopped before deployment because it was slow; the final mini image was built natively on the Mac mini from the same published source revision.

## Remaining verification

- Owner decryption was not verified because neither fossil has a node-local age identity file and no browser unlock flow was performed. Keep the Syncthing ignore unchanged until owner access is confirmed.
- The default native Syncthing `config.xml` on both Macs did not list a Rawrarr shared folder. No `.stignore` was created or modified because the applicable Rawrarr folder path was not present in the configured folders. Identify the active Rawrarr Syncthing folder/instance, verify owner decryption, then apply only the root-level `/config.aged` ignore on both nodes. App metadata/artwork Syncthing behavior was not changed by this deployment.
- GitHub issue #100 remains open for owner-decryption, Syncthing exclusion, and isolated convergence acceptance follow-up.

## Status and remaining work

The approved protocol is in [metadata-peer-repair-protocol.md](../docs/metadata-peer-repair-protocol.md). RPC, signed journals, and periodic Merkle reconciliation are merged and deployed to both fossils; the Mac mini has its own Valkey service. Live config ciphertext converged from the MacBook through peer RPC and both caches match it. The mini coordinator and Tailscale route remain healthy. Owner decryption and the exact Syncthing folder mapping still need verification before excluding `/config.aged`; keep app metadata/artwork behavior unchanged.
