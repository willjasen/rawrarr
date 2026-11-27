# Rename local state and define event data domains

Issue: [#486](https://github.com/willjasen/rawrarr/issues/486)

## Objective

Use `/local` for node-local persistent state. Define `/evt` as the event-data root, with `/evt/fossil` reserved for authenticated RPC replication between authorized nodes and `/evt/boneyard` for explicitly selected Syncthing-replicated media and other large files.

## Current state and assessment

The application currently uses `/data` for node-local files and `/shared` as a Syncthing root. Metadata and artwork are written beneath that root. Some app configuration repair already uses authenticated peer RPC/Merkle, but shared config files and many metadata assets still use filesystem paths and Syncthing. The current Syncthing companion mounts a whole host directory at `/shared`. This runbook establishes the target contract; it does not claim that new replication behavior is implemented or change live services.

## Safe adoption design

1. Publish an application release that understands both legacy and new paths. Continue reading `/shared` while `/evt/fossil` and `/evt/boneyard` are populated.
2. Keep existing named-volume names and mount them at `/local` for node-local state. This preserves existing volume contents. Keep `RAWARR_DATA_DIR` as a compatibility alias while new configurations use `RAWARR_LOCAL_DIR`.
3. Keep `/shared` mounted during mixed-version rollout. Before moving data into `/evt/fossil`, define the RPC object manifest, authorization, signatures, conflict ordering, tombstones, retry/reconciliation, and a single-writer or fencing strategy. Copy only with verified digests and retain the source; do not merge simultaneous writes with blind overwrite.
4. Copy explicitly selected large-file trees to `/evt/boneyard`, verify manifests/digests, configure the Syncthing folder to that subtree on each peer, and verify each node independently. Do not sync `/evt/fossil` through Syncthing after RPC becomes authoritative.
5. Switch readers and writers only after all authorized peers run compatible code and verified data is present. Keep legacy reads through the rollback window. Remove legacy mounts/data only in a separately reviewed cleanup after backup and peer verification.

Rollback before writer cutover is to resume legacy `/shared` reads and writes; copied event trees remain untouched. After cutover, rollback requires reverse compatibility or a quiesced, verified copy back. Never delete either copy automatically.

## Verification and remaining work

`docs/data-domains.md` records the target split and compatibility boundary. No application paths, Compose mounts, or live persistent volumes were changed in this design increment. The current app still uses `/data` and `/shared`; implementing the RPC path for all metadata/config objects and the `/evt/boneyard` Syncthing topology requires a separate code and integration-test increment before deployment.
