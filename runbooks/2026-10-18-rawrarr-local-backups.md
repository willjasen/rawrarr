# Plan local backups of Rawrarr shared data

## Objective and initial state

Record the requested backup behavior: copy the Rawrarr Syncthing shared folder to a local subscriber node, keep backups local to that node, and track the backup destination in the main Rawrarr configuration. Reviewed [data domains](../docs/data-domains.md), [node types](../docs/node-types.md), [device architecture](../docs/device-architecture.md), and the shared age-config notes.

## Design recorded

- Put the enabled state, target `device_id`, and target-local destination path in age-encrypted `config.aged`.
- Let only the designated node create backups. Never automatically fail over or create backup copies on every peer.
- Keep backups outside `/shared` and `/media`; do not sync them, send them through Headscale/relay, or include media or local PostgreSQL datadirs.
- Prefer age-encrypting an archive to public recipient(s), since the shared folder may contain operational files that are not individually age-encrypted. The backup operation must never access a private identity.
- Require verified temporary-write then atomic rename; keep mutable execution status node-local. Restore is explicit and staged separately from backup creation.
- Added the [local backup design](../docs/local-backups.md), data-domain placement, and runbook index entry.

## Verification and remaining work

Planning documentation only. No backup archive, scheduler, UI, filesystem path, or container was created or changed. Archive encryption, schedule/retention defaults, path-selection flow, archive format, and restore behavior remain open before implementation. `git diff --check` and relative-link validation passed.
