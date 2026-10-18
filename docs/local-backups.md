# Local backups of Rawrarr shared data

## Goal

Let an instance keep recoverable backups of its Syncthing-shared Rawrarr folder on one designated subscriber node. The backup destination is configured in the age-encrypted main configuration, but backup files stay only on that node and are not placed under `/shared`, replicated by Syncthing, uploaded to the service node, or sent through Headscale/relay traffic.

This scope backs up the shared folder only. Node-local PostgreSQL datadirs, sessions, passkey challenges, and media files are separate backup domains and are not included.

## Configuration and execution model

The shared configuration should contain a backup target with:

- `enabled`;
- the designated Rawrarr `device_id`;
- the destination path as configured on that device;
- manual-only operation for the first version; retention is not selected yet.

Every node can read the target after its normal age-config unlock. Only the node whose local `device_id` matches the configured target may execute backups. The absolute destination path is meaningful only on that device; other nodes display it as configuration/status but must not try to create it. If the target device is offline, report that no backup ran and resume only when that node returns. Do not silently switch to another node or create copies on all peers.

The destination must be node-local and outside `/shared`, its resolved/symlinked descendants, and `/media`. Reject paths that could create recursive backups or put media into the shared-data backup set. Write each archive to a temporary file in the destination filesystem, verify it, then atomically rename it into the backup set. Record completion time, size, and verification state in node-local operational state; do not synchronize a mutable backup index through Syncthing.

## Backup contents and protection

Create a consistent snapshot of the shared folder, including `config.aged`, age-encrypted metadata/artwork (`.aged`), the public age recipient, and shared operational logs. Preserve relative paths and file bytes. Skip transient Syncthing internals and incomplete temp files. Never follow symlinks outside the shared root.

Because some shared files (for example logs) may not themselves be age-encrypted, encrypt every finished backup archive to the instance's public age recipient(s). The backup task needs only public recipient material; it must never request, persist, or transmit a private age identity. The user must retain an authorized private identity separately to restore an encrypted backup. Do not record sensitive filenames in plaintext archive indexes.

Do not delete, overwrite, or age-re-encrypt source files during backup or restore. Restore should be an explicit, user-confirmed operation to a separate staging directory; verify the archive before presenting a replacement/copy-back step.

## UI and status

The Rawrarr settings UI should show the configured target device, destination, manual-only mode, last success/failure, and whether this node is the designated executor. Provide a **Back up now** action only on the target node. Offer **Verify** and restore discovery for existing archives; keep restore separate from backup creation. Errors should identify actionable filesystem problems without printing secrets or file contents.

## Open decisions before implementation

- Manual-only backups are selected for the first version; no automatic schedule or retention deletion.
- Age-encrypting each archive to public recipient(s) is selected.
- Choose retention behavior and user-driven deletion policy before implementing cleanup; until then, never automatically delete older archives.
- Decide how the UI selects the target device/path when the path exists only on that device, and how a device replacement/reprovision updates the target.
- Decide whether backup archives are compressed and select an archive/manifest format with integrity checks and safe path handling.
- Confirm the restore user experience and whether Rawrarr will restore automatically or initially provide verified archives for manual recovery only.
- Decide whether to include future portable passkey/user records when they are added to `/shared`; never include the live PostgreSQL datadir as a substitute.

## Related architecture

- [Data domains](data-domains.md) — what belongs in `/shared` versus node-local data and media.
- [Node types](node-types.md) — subscriber storage/shard nodes and operator service-node boundary.
- [Device architecture](device-architecture.md) — stable instance/device identities and local-versus-shared app data.
- [Subscription model](subscription-model.md) — data recovery remains available after entitlement expiry.
