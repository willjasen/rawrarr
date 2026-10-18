# Store Rawrarr config as shared age ciphertext

## Objective and initial state

Put Rawrarr's durable settings in the host folder used for its Syncthing share and ensure the config is age encrypted. The code already encrypted settings, but used `config.age` under a Docker named volume, which was not mounted into Syncthing. The sharing test already used age-encrypted artwork with a `.aged` suffix; see [encrypted config](2026-10-17-rawrarr-age-encrypted-config.md), [Syncthing peer sync](2026-10-17-rawrarr-syncthing-peer-sync.md), and [storage-less web node](2026-10-17-rawrarr-storage-less-web-node.md).

## Changes

- Store settings as `config.aged` under `RAWARR_SHARED_DATA_DIR` (default `data/shared`).
- Docker keeps node-local app data and PostgreSQL in local volumes, and bind-mounts the host Syncthing folder at `/shared` using `RAWARR_SHARED_DATA_PATH` (default `./shared-data`). Rawrarr reads/writes shared settings through `RAWARR_SHARED_DATA_DIR=/shared`.
- Share-folder ciphertext and parent directories are readable/traversable by Syncthing. Plaintext is encrypted from memory and never saved in the shared folder. Removed plaintext `RADARR_URL` / `RADARR_API_KEY` bootstrap configuration so the app's Radarr settings have one encrypted persistence path.
- User direction: user/passkey records should also be shared as portable age-encrypted records, while PostgreSQL datadirs and sessions remain local. Passkey-first login on a new node is preferred; its secure bootstrap flow is unresolved because passkey records are encrypted in the share.
- Extend the config API test to assert the `.aged` path and ciphertext header. Extend the OrbStack Syncthing integration test to sync an age-encrypted config artifact and verify byte-identical ciphertext with no plaintext in the replica.

## Verification and remaining work

The Docker release image built successfully. The targeted Rust test passed, covering config/artwork age ciphertext, plaintext absence, decryption, shared-file permissions, and rejection of a wrong age identity. The OrbStack test passed with separate Syncthing web/storage containers, matching metadata SHA-256, and byte-identical encrypted `config.aged`; temporary containers/network/files were cleaned up. `docker compose config -q`, Python bytecode compilation, and `git diff --check` passed. The updated `rawrarr-local` app is running at `http://localhost:8084`; `/health` returned `ok`, `/auth` returned HTTP 200, `/shared` is writable by the app UID, and inspection confirmed `/shared` bind-mounts the repo's `shared-data` directory while `/data` and PostgreSQL remain named volumes. The earlier local app-data volume was empty, so no config migration was needed.

The config API integration test was not run: it needs a release binary and an isolated disposable PostgreSQL database ending in `_test`. The current `/shared` mount is not yet a Syncthing-managed folder; set `RAWARR_SHARED_DATA_PATH` to the local Syncthing folder before expecting app data to replicate. PostgreSQL passkeys/user records remain node-local today despite the product requirement that portable user/passkey records share; new-node passkey-first bootstrap needs a secure design before implementing that replication.
