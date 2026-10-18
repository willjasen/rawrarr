# Dedicated shared filename secret

## Objective and initial state
Add a shared secret exclusively for opaque metadata filename derivation in the main age-encrypted config. Existing files keep their current names; HMAC naming and migration are a separate step. The local service has a remembered node-local identity and an existing encrypted config. Many unrelated working-tree changes predate this task.

Related: [shared age config](2026-10-17-rawrarr-shared-age-config.md), [local Radarr mirror](2026-10-18-local-radarr-config-mirror.md).

## Actions and verification
- Added `metadata_filename_key`: 32 OS-random bytes encoded as 64 lowercase hex characters, stored exclusively in the main encrypted config. Its sole reserved use is metadata filename derivation; no filename consumer is implemented yet.
- Legacy configs are initialized when unlocked/read; new configs on save. Valid existing keys are preserved, invalid values fail validation instead of rotating silently. In-process config initialization and writes are serialized. Provision once and synchronize before other nodes initialize legacy copies.
- Radarr saves and removal preserve the key. Removal now clears Radarr fields instead of deleting the main config, and startup/unlock accept that secret-only config. The optional local plaintext mirror contains Radarr fields only. Configuration API exposes only a configured boolean. Secret-bearing config no longer derives Debug; its key is zeroized on drop.
- All 18 Rust tests passed, including random generation, validation, real age-encrypted legacy migration, stable peer reads, changed Radarr credentials, clearing/re-adding Radarr, and mirror exclusion. Node suite 14/15 passed with the existing health-page capitalization failure; the separate setup browser test passed. Python integration and real multi-node synchronization were not run.
- Before deployment, copied ciphertext to `/data/config-before-filename-key-2026-10-18.aged` with owner-only permissions, outside the synchronized tree. No identity or plaintext secret was printed or saved by the verification.
- Rebuilt with `RAWARR_POSTGRES_PASSWORD_FILE="$HOME/Library/Application Support/rawrarr/orbstack-postgres-password" docker compose -p rawrarr-local up -d --build rawrarr`, after verifying ownership of port 7297. The remembered identity initialized the existing encrypted config on startup.
- Decrypted the live config and rollback copy only in verification-process memory. Confirmed a valid 256-bit key, identical pre-existing settings, and exclusion from the local Radarr mirror. Container is healthy and `/health` returns HTTP 200.
- All task changes remain uncommitted: source changes depend on extensive pre-existing config-mirror/startup changes, and concurrent tasks actively staged quality-profile and storage-node-plan work during completion. Removed only this task's two newly staged files from the shared index; left other staged work untouched. Branch is main; no task commit or push was made, and no remote/upstream is configured. HMAC filenames and migration remain planned.

## Recovery
Retain an encrypted pre-change backup outside the synchronized tree before deployment. Do not rotate or discard the filename key once opaque filenames use it. Provision on one node and synchronize the resulting encrypted config before initializing other nodes.
