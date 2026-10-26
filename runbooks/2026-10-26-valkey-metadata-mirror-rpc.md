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

## Verification

- Read the current shared-cluster plan, device-signing design, Valkey artwork-cache implementation, and prior Valkey/peer-sync runbooks.
- Read-only SSH/Docker inspection confirmed the MacBook fossil and Valkey are healthy; config ciphertext exists there. The Mac mini fossil is healthy, has no Valkey service, and lacks config ciphertext. No container was restarted and no environment values, config contents, or secrets were printed.
- Read-only checks confirmed `/data/radarr-config.json` is absent on both live nodes; the MacBook config file has the expected age ciphertext prefix. No plaintext configuration was read or printed.
- Confirmed both Macs run native Syncthing and neither `/shared` root has `.stignore`. No ignore file was written. The official Syncthing guidance says `.stignore` is local to each synchronized folder and is not propagated to other devices; add and verify the root-level `/config.aged` rule on both nodes only after both run the RPC-capable published release.
- Confirmed issue #100 is open and updated its title and acceptance criteria.
- No tests or source behavior changes have been made yet.

## Status and remaining work

The protocol draft is in [metadata-peer-repair-protocol.md](../docs/metadata-peer-repair-protocol.md). The project plan requires protocol review before durable RPC implementation or live Syncthing exclusions; those changes have not started. After review, implement and validate the isolated two-fossil path, then publish to `main` before adding Valkey to the mini fossil or excluding `config.aged` from Syncthing. Preserve `/shared/metadata`, artwork, the mini coordinator, and both fossils' node-local state.
