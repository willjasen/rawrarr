# Implement the “eventually” shared-data layout

## Objective and initial state

Implement issue [#211](https://github.com/willjasen/rawrarr/issues/211): store app metadata under `/shared/in`, publish explicitly selected large files under `/shared/out`, and preserve existing internal data with a non-destructive migration path. Both subtrees remain under the existing Syncthing `/shared` root. Databases and other node-local state remain local.

Reviewed [product direction](../docs/product-direction.md), [product design](../docs/DESIGN.md), [data domains](../docs/data-domains.md), [technical status](../docs/technical-status.md), and prior [data-domain runbook](2026-10-18-rawrarr-data-domains.md).

The user clarified the intended directory structure: `/shared/in` for app data such as Radarr metadata shared between fossils, and `/shared/out` for larger files such as selected media collections. Both use Syncthing. The initial implementation stored metadata under `/shared/metadata` and used a separate share for media. No live internal environment was changed in this task.

The user further clarified the data boundary: Radarr configuration data (URLs, API keys, preferences) belongs to the separate RPC/Merkle work in another branch; Radarr metadata (for example, retrieved records and artwork) remains Syncthing-replicated. The user requested documentation/design clarification only and asked that implementation wait for the other task to finish.

## Actions

- Changed Rawrarr's default metadata path to `/shared/in/metadata` and updated the container `/metadata` alias. Vault unlock copies files missing from the new path from `/shared/metadata`, leaves legacy files in place, and never overwrites an existing destination file.
- Added `migrate_shared_metadata.py` for an explicit dry-run/`--apply` migration before rollout; it preflights conflicts and capacity and preserves the legacy source.
- Added `publish_shared_output.py` to dry-run and copy one explicitly selected source tree under `/shared/out`; it refuses overlap, symlinks, conflicting files, and insufficient space, and does not remove or overwrite files.
- Updated product, data-domain, security, technical-status, user, and implementation docs to distinguish app ciphertext under `/shared/in` from large published files under `/shared/out`, and to describe the existing separate media share as current behavior.
- Clarified in product direction, design, data-domain, security, and technical-status docs that app configuration data and app metadata are separate sync domains: RPC/Merkle for config data; Syncthing for metadata. This is documentation only; no behavior was changed for the other task's work.
- Added script documentation and regression coverage. Issue #211 tracks acceptance criteria.

## Verification

- `git diff --check` passed.
- `python3 scripts/tests/test_shared_tree_layout.py` passed.
- Focused Rust path and migration tests are pending.
- No internal hosts, running Syncthing folders, or shared files were changed.

## Internal migration readiness review

Review of `AGENTS.md` confirms deployment must use the exact published `main` revision, preserve `/shared`, and deploy only affected application services under their own locks. The internal Syncthing service already binds the host shared-data directory as `/shared`; no new Syncthing folder is required for these subdirectories.

For an orderly migration, treat `/shared/in/metadata` as one shared-tree migration on the canonical host path, not one independent copy per fossil. Before writing, identify and record the exact `RAWARR_SHARED_DATA_PATH` used by the mini coordinator's Syncthing and the MacBook fossil/Syncthing, confirm both resolve to the intended peer copies, list currently configured Syncthing folder IDs/devices and peer completion state, and check free space on every peer. The owner explicitly approves every existing shared-root peer for the data. Make a protected backup of the shared tree outside `/shared`, and record file count, bytes, and manifest for legacy and destination trees. This review did not read live hosts or Syncthing state; those facts remain to be collected during the authorized migration.

1. Publish and merge the complete code change, then follow `AGENTS.md` to deploy the exact published `main` revision. Keep the existing `/shared` bind mount and Syncthing folder ID unchanged. Since this changes Rust subscriber-node behavior, deploy the affected subscriber (`fossil`); coordinator code is not affected by metadata-path behavior.
2. Before the first affected process starts or unlocks its vault, stop metadata writers on both fossils and pause Syncthing on all peers for the shared folder. Verify no active writer or pending scan is mutating legacy metadata. Keep services stopped through migration and initial synchronization.
3. On the canonical existing shared-data root, run `python3 scripts/migrate_shared_metadata.py --shared-dir '<verified host shared-data path>'` and save its dry-run counts, byte totals, and manifest in the protected operational record (never put user data or paths with private information in Git). Review disk capacity and resolve any differing-file conflicts manually without overwriting either copy. Then run the same command with `--apply` exactly once on that root. The script preserves source files, refuses conflicting content and symlinks, checks free space, and atomically creates only missing destinations.
4. Verify the destination tree against the source: run the dry-run again (it should report zero files to copy), compare per-file SHA-256 manifests on the canonical root, then resume Syncthing and wait until each configured peer reports the shared folder up to date. On each peer, verify paths, counts, byte totals, and hashes. Do not independently run `--apply` on replicas while Syncthing is reconciling.
5. Deploy/restart the subscriber code only after the synchronized target is complete. On each fossil, verify `/metadata` resolves to `/shared/in/metadata`, unlock the age vault, confirm metadata reads/writes use the new subtree, and check service health. Keep `/shared/metadata` intact through an observation period and until the rollback window is closed.
6. For rollback before new-path writes, stop affected subscribers and Syncthing, point the image/configuration back to the prior release (whose `/metadata` link targets `/shared/metadata`), and resume. If any new writes have reached `/shared/in/metadata`, first reconcile both trees with a conflict-reviewed, no-overwrite procedure while writers are stopped; do not simply switch the symlink and strand newer records. Keep the old tree and backup until owner acceptance.

The migration script does not configure or pause Syncthing, stop application writers, create a protected backup, or verify peer state; those are explicit operator preconditions. Publishing `/shared/out` is a separate opt-in operation: confirm each current peer is approved for the selected plaintext files and has enough capacity before using `scripts/publish_shared_output.py --apply`.

## Final state and remaining work

New Rawrarr writes use `/shared/in/metadata`. Existing `/shared/metadata` data is preserved and can be migrated explicitly or copied forward during vault unlock. Selected large files can be published under `/shared/out` for Syncthing replication to peers already approved for `/shared`. The separate media-share workflow and existing live internal folders remain unchanged until a separately scheduled migration/deployment. Root `config.aged` and node-local `/data` and PostgreSQL are unchanged. The ordered internal migration procedure above is documented but not executed or live-verified.
