# Rename local state and define event data domains

Issue: [#486](https://github.com/willjasen/rawrarr/issues/486)

## Objective

Use `/local` for node-local persistent state. Define `/evt` as the event-data root, with `/evt/fossil` reserved for authenticated RPC replication between authorized nodes and `/evt/boneyard` for explicitly selected Syncthing-replicated media and other large files.

## Current state and assessment

Initially, `/data` held node-local files and `/shared` was mounted as a Syncthing root. Config and metadata had several active writers there. Existing config repair used signed peer RPC/Merkle for config only. No live services have been changed.

## Safe adoption design

1. Runtime prefers `RAWARR_LOCAL_DIR` and recognizes legacy `RAWARR_DATA_DIR`; the Compose named volume is mounted at `/local` to preserve its contents.
2. Runtime creates `/evt/fossil`, copies only missing metadata/config files into it, and retains the source. Conflicting destination data is never overwritten.
3. Compose mounts a separate persistent event volume at `/evt`. `RAWARR_EVT_WRITE_MODE=evt` enables fixed-path metadata and config storage under `/evt/fossil`, signed metadata revision exchange, bounded content-addressed ciphertext transfer, conflict ordering, and tombstones over the existing authorized peer RPC. The image and generated Compose remain in `legacy` mode by default for mixed-version rollout.
4. Syncthing now requires `RAWARR_BONEYARD_DATA_PATH` and mounts only that selected path at `/evt/boneyard`; the old whole-tree mount must be configured deliberately during adoption.
5. Switch readers/writers only after all authorized peers run compatible code and verified data is present. Remove legacy data only in a separately reviewed cleanup after backup and peer verification.

Rollback before writer cutover is to resume legacy `/shared` reads and writes; copied event trees remain untouched. After cutover, rollback requires reverse compatibility or a quiesced, verified copy back. Never delete either copy automatically.

## Actions and verification

Implemented: `/local` path preference with the old environment variable retained as an alias; `/evt` paths and compatibility copy that retains sources and destination conflicts; signed fixed-path metadata revisions with content-addressed encrypted object transfer, bounded bodies, deterministic conflict resolution and tombstones; `/evt/fossil` config placement in event mode; split Compose volumes; and a Syncthing boneyard mount scoped to the selected directory. Per-object observed digests prevent an offline node's stale file from being re-published as a new edit after it has imported peer state.

The Compose/Syncthing path audit found that the Syncthing companion's selected boneyard host directory was not mounted into the Rawrarr services, and the local bootstrap still configured its folder at `/shared`. The app and Syncthing now bind the same `RAWARR_BONEYARD_DATA_PATH` to `/evt/boneyard`. Both also retain a separate `/shared` compatibility bind while generated writers default to legacy mode; the local bootstrap preserves that folder and adds `/evt/boneyard`. Syncthing never mounts `/evt/fossil`. Valkey remains an in-memory, disposable encrypted cache with persistence disabled and no data volume.

Checks run:

- `cargo test --locked --offline --manifest-path rust-arr/Cargo.toml config_repair --no-fail-fast` — 13 passed, including the two-node test. That test runs isolated node processes in both directions against authorized mock peer RPC endpoints.
- `cargo test --locked --offline --manifest-path rust-arr/Cargo.toml secure_config:: --no-fail-fast` — 37 passed.
- Node.js 26.13.1 container: `test_rawrarr_brand_case.mjs` and `test_rawrarr_subscriber_compose.mjs` — 5 passed.
- `test_compose_layout.py` — 8 passed; `test_setup_local_syncthing.py` — 5 passed, including rejection of overlapping host paths.
- Node.js 26.13.1 container: subscriber Compose test — 4 passed.
- `docker compose config --format json` validated local, metadata, storage, and Syncthing Compose files with temporary fixtures; resolved `/shared` and `/evt/boneyard` host sources matched between each app service and Syncthing. Valkey had no volume and persistence flags remained disabled.
- The documented full regression runner was run earlier. It used Node.js 26.13.1; Rust findings were fixed and the focused Rust groups above pass. Playwright had two existing browser navigation timeouts (crypto ENS login and artwork); the full runner has not been rerun after fixes.
- `git diff --check` passed.

The host still has Node.js 24.22.3, so run Node-dependent project checks through the documented Node.js 26 test image; the project `.nvmrc` and test clone target Node 26. No host runtime was changed. The two-node RPC test covers both directions, reconnect repair, object transfer, deterministic conflict resolution, tombstone propagation, and retry idempotency using mock peer endpoints. The Compose config check confirms mount resolution and cache persistence settings but does not run application services, Valkey requests, or actual Syncthing indexing/transfers. The broader service integration described in [the protocol](../docs/metadata-peer-repair-protocol.md) remains a deployment gate. The live services were not changed. Rollback before cutover is to retain legacy mode. After cutover, reverse copy and writer coordination are required; no automatic deletion is provided.
