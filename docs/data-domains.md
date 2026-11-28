# Rawrarr data domains

Rawrarr uses stable container paths to describe where each kind of data belongs. The paths are the same on every node; a node only mounts the domains it can serve. `/evt` is the event data root. `/evt/fossil` holds metadata and authorized configuration ciphertext exchanged through the fossil RPC protocol. `/evt/boneyard` holds media folders replicated by Syncthing. `/local` is node-local persistent state. Only explicitly owner-approved nodes may exchange RPC data or be configured as Syncthing peers; node enrollment, Tailscale membership, or network reachability alone does not grant data access.

The data domains separate managed app configuration, portable metadata, media, and local state. Fossil metadata and authorized managed-app configuration use bounded, authenticated RPC with signatures and conflict reconciliation; this data is excluded from Syncthing. Syncthing replicates the boneyard tree, including media folders selected for sharing. Existing deployments migrate legacy paths repeatably while preserving their content. During rollout, readers support old paths and writers use the selected new path after migration. `/local` and PostgreSQL remain node-local.

The node-side app-key grant verifier and bounded app-config ciphertext primitive are preparatory. They stage app ciphertext under `/evt/fossil/apps/<app-instance-id>/config.aged` using an installed app identity, but no owner-side grant delivery, live settings integration, or app-specific RPC replication path is connected. The group-encrypted app identity registry is not an authorization source.

The node-side app-key grant verifier and bounded app-config ciphertext primitive are preparatory. They stage app ciphertext under `/evt/fossil/apps/<app-instance-id>/config.aged` using an installed app identity, but no owner-side grant delivery, live settings integration, or app-specific RPC replication path is connected. The group-encrypted app identity registry is not an authorization source.

| Domain | Container path | Present on | Persistence and sync | Contents |
|---|---|---|---|---|
| Local state | `/local` | Each node | Node-local persistent volume; never synced | Device identity, runtime state, credentials, and node-local databases or caches |
| Fossil data | `/evt/fossil` | Authorized nodes | Authenticated RPC replication; never Syncthing | Encrypted app metadata and authorized managed-app configuration objects |
| Staged app configuration | `/evt/fossil/apps/<app-instance-id>/config.aged` | Local staging only; RPC exchange is not active | App-recipient age ciphertext; does not use Syncthing | Versioned app JSON that requires the matching installed app key |
| Staged app configuration | `/evt/fossil/apps/<app-instance-id>/config.aged` | Local staging only; RPC exchange is not active | App-recipient age ciphertext; does not use Syncthing | Versioned app JSON that requires the matching installed app key |
| Boneyard | `/evt/boneyard` | Nodes selected for shared media | Syncthing folder; only approved peers | Media folders and files selected for replication |
| Media source | `/media` | Storage nodes that can read media | Host media mount, read-only in Rawrarr; never copied automatically | Original movie, TV, music, book, subtitle, and other library files |
| Database | PostgreSQL data volume | Coordinator | Node-local database volume; never Syncthing | Coordinator authentication and application state |

## Placement rules

- Keep private age identities, passkeys, plaintext credentials, PostgreSQL datadirs, and node-local application state in `/local` or the appropriate node-local database volume. Never place these in `/evt`.
- The coordinator owns the user-facing web/auth stack and PostgreSQL. Fossil nodes exchange only owner-authorized encrypted data. Storage nodes may hold approved shared media without coordinator services.
- Fossil RPC peers must be authorized by the owner and validate peer identity, bounded requests, signatures, and deterministic conflict handling. Enrollment or tailnet reachability alone is insufficient.
- Syncthing is reserved for `/evt/boneyard` media folders. It must not replicate `/evt/fossil`, `/local`, private identities, credentials, or live databases. Use distinct folder IDs and explicit peer approval for each selected media tree.
- Keep Syncthing GUI and peer traffic on their configured interfaces; the application and Syncthing companion must mount the same host boneyard directory at `/evt/boneyard`. Do not mount fossil data into Syncthing.
- Valkey is an ephemeral cache. Disable snapshot and append-only persistence; do not mount node data directories into it.
- Migrate existing `/data` contents to `/local` and `/shared` contents to the matching `/evt` domain without deleting source data until the destination is verified. The migration must be repeatable and preserve identities and credentials.
- Do not write media bytes, private age identities, plaintext configuration, sessions, challenges, or a live PostgreSQL datadir under `/evt`.

## Container mapping

The application image exposes `/local`, `/evt/fossil`, and `/evt/boneyard` as distinct paths. Compose must bind each to its corresponding host directory. Syncthing mounts only the boneyard host directory at `/evt/boneyard`; its legacy `/shared` mount may remain temporarily for compatibility during migration. Valkey has no persistent data volume.

This registry defines placement and sync boundaries. It does not imply that a web node can remotely read a storage node's `/media` tree or that cross-node catalog aggregation is available.

## Retained legacy behavior during migration

Older releases used `/shared` as a Syncthing root, with `/shared/in/metadata` for encrypted media metadata and `/shared/out` for selected published files. Older local state used `/data`. Keep compatibility reads during staged upgrades; do not treat old paths as the active placement contract. Root `config.aged` was previously synchronized and is not an authorization boundary by itself. The new RPC path must enforce approved peers and preserve conflict/tombstone semantics.

The prior [local backup feature](local-backups.md) stores a target device ID and local destination in shared encrypted configuration. Keep backup archives outside `/evt` so Syncthing cannot replicate them back into the instance.

Older releases kept metadata in `/shared/in/metadata` and published files in `/shared/out`; vault unlock copied missing legacy metadata without deleting its source. These paths remain compatibility sources during migration, not current placement targets. Portable metadata should eventually reach offline peers, but that does not make every database row or setting shared.

For historical media metadata structure and encryption details, see [metadata architecture](metadata-peer-repair-protocol.md) and the linked runbooks. Those descriptions document legacy compatibility and do not change the current `/evt/fossil` and `/evt/boneyard` placement rules.
