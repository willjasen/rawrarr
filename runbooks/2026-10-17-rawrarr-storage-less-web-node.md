# Rawrarr storage-less web node

## Objective and initial state

Record the user's clarification that a web node without media mounts should still serve the Syncthing shared folder containing Rawrarr application data. Existing notes distinguish scanner and web roles but did not say the web node participates in the shared data folder; see [device architecture](../docs/device-architecture.md), [shared instance identity](2026-10-17-rawrarr-shared-instance-id.md), and [Syncthing peer sync](2026-10-17-rawrarr-syncthing-peer-sync.md).

## Architecture recorded

- The web/control node may have no media mounts, but keeps a local Syncthing replica of the shared Rawrarr application-data folder and serves that state through the UI/API.
- Storage/scanner nodes keep media locally and synchronize Rawrarr catalog/metadata state; media bytes are not copied into the shared folder by default.
- Syncthing is file replication, not multi-writer database coordination. PostgreSQL's live data directory remains local. Shared mutable records need portable age-encrypted files/events and application-level reconciliation.
- Passkey credentials and sessions remain local authentication state unless a separate credential-replication design is chosen.

## Verification and remaining work

Checked the revised architecture and product-direction text for consistency with the stable shared `instance_id`, node-specific `device_id`, age-encrypted shared data, and 24-hour trial direction. No implementation or running service was changed. Before implementation, define the exact shared-folder schema, conflict-resolution rules, and whether any PostgreSQL-backed domain data must be exported into portable synchronized records.
