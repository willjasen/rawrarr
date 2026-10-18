# Rawrarr data domains

Rawrarr uses stable container paths to describe where each kind of data belongs. The paths are the same on every node; a node only mounts the domains it can serve. `/shared` remains the root of the Syncthing folder. `/metadata` is an in-container alias for `/shared/metadata`, not a second sync folder.

| Domain | Container path | Present on | Persistence and sync | Contents |
|---|---|---|---|---|
| Media | `/media` | Storage nodes that can read media | Host media mount, read-only in Rawrarr; never copied by Rawrarr or Syncthing | Original movie, TV, music, book, subtitle, and other library files |
| Metadata | `/metadata` | Every node | Alias of `/shared/metadata`; age-encrypted files sync as part of the shared folder | Library JSON sidecars in `library/`; encrypted artwork in `movies/` |
| Shared app state | `/shared` | Every node | Syncthing replicated; encrypt private payloads with age before writing | `config.aged`, public `age-recipient`, and the metadata domain |
| Local operational state | `/data` | Each node | Node-local persistent volume; never synced | Device ID and local runtime state |
| Database | PostgreSQL data volume | Coordinator | Node-local database volume; never put its live datadir in Syncthing | Current passkey/authentication database and local projections |
| Shared-folder backups | Configured node-local destination | One designated subscriber node | Stored outside `/shared`, never Syncthing-replicated; destination device/path is recorded in age-encrypted `config.aged` | Verified snapshots of the shared Rawrarr folder only; excludes node-local database and `/media` |

## Placement rules

- The coordinator owns the user-facing web/auth stack and PostgreSQL. Storage and shard nodes can hold authorized shared ciphertext without those services; `/metadata` remains an alias inside `/shared`.
- Only storage capability mounts local media read-only under `/media`. A coordinator can optionally also have storage capability. A shard has no media mount; multi-library/cross-node catalog behavior remains planned.
- `/metadata` is a classification and access path, not a new sync root. All metadata remains physically under `/shared/metadata`, so every synchronized node sees the same ciphertext and new nodes do not need a second Syncthing folder.
- Metadata files use `.aged`; names and directory structure should avoid leaking media titles or absolute host paths. The current library sidecar filename is a stable hash, and the relative media path stays inside ciphertext.
- Do not write media bytes, private age identities, plaintext configuration, sessions, challenges, or a live PostgreSQL datadir under `/shared` or `/metadata`.
- User passkeys and browser sessions belong to the coordinator; storage-only and shard nodes use device identity instead. Portable records for future multiple-coordinator use remain undecided and are not required for this deployment.

## Container mapping

The Docker image sets `RAWARR_METADATA_DIR=/metadata` and makes `/metadata` a symlink to `/shared/metadata`. Native runs can omit that variable; Rawrarr then uses `$RAWARR_SHARED_DATA_DIR/metadata`. Keep an override inside the shared folder if setting it manually. Compressed operational logs are stored under `$RAWARR_SHARED_DATA_DIR/logs/<device-id>/` and synced as application data. `RAWARR_LIBRARY_PATH` identifies the media root and should be `/media` in a storage container.

The local Compose example bind-mounts `$HOME/Library/Application Support/Rawrarr/shared-data` at `/shared` by default so the shared data is persistent and directly accessible in Finder. Override the host directory with `RAWARR_SHARED_DATA_PATH` when needed. Keep this directory outside the source checkout and never place private age identities or PostgreSQL data in it.

This registry defines placement and sync boundaries. It does not imply that a web node can remotely read a storage node's `/media` tree or that a metadata reader, catalog projection, or remote scan-command channel already exists.

The planned [local backup feature](local-backups.md) stores a target device ID and its local destination in the shared encrypted configuration. Only that node writes backup archives, and the destination must remain outside `/shared` so backups do not sync back into the instance.

Movie text metadata also caches `qualityProfileId` and `qualityProfile.name`. Movies reads these encrypted shared values first, resolves missing names in one Radarr quality-profile request, and merges updates without replacing other metadata. A changed assigned ID invalidates the cached name. Reading existing ciphertext requires the unlocked subscriber vault; unreadable metadata is not overwritten.
