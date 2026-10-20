# Store and restore Radarr recovery data through Syncthing

## Objective and decisions

Keep Radarr recoverable from the Rawrarr shared folder so the owner can deploy a Radarr container on a chosen metadata or storage node and restore its application state there. The owner deploys the container; Rawrarr does not control Docker or copy media files.

- Create a Radarr native manual backup when the age vault and Radarr connection are available, then repeat every six hours.
- Encrypt the single latest archive with the shared public age recipient at `/shared/metadata/radarr/recovery.zip.aged`. Preserve the existing snapshot if capture or encryption fails.
- Keep Rawrarr's scraped movie metadata and artwork in their existing age-encrypted shared folders; these are separate from the native Radarr backup and remain available after restore.
- Restore only after the owner has deployed a target Radarr. The helper reads the private age identity from a node-local file, validates the decrypted ZIP, prompts for the target API key, and uploads to Radarr's restore endpoint.
- Do not copy or modify `/media`. The target must use the same Radarr root-folder paths as the backup or the owner must repair paths after restart.

## Implementation

- Added the Radarr `Backup` command, status polling, manual backup listing, bounded archive download, and age-encrypted shared persistence to the non-coordinator Radarr worker.
- Triggered a fresh attempt after unlocking the age vault or saving Radarr settings. The worker retries on its six-hour cadence and logs generic errors without upstream response bodies or credentials.
- Added `scripts/restore_radarr.py` for an owner-driven restore on the selected node. It limits the archive to 512 MiB, rejects unsafe ZIP paths/symlinks and oversized archives, allows HTTP only for loopback, and avoids putting API keys or identity contents in process arguments.
- Documented the data path, secret boundary, and manual deployment/restore steps in the user guide and data/security documentation.

## Verification

- `python3 scripts/tests/test_rawrarr_radarr_recovery.py`
- `python3 -m py_compile scripts/restore_radarr.py scripts/tests/test_rawrarr_radarr_recovery.py`
- `cargo test --locked --offline --manifest-path rust-arr/Cargo.toml radarr_backup_tests` was attempted on the host but rustc could not start because its LLVM dependency `libz3.5.1.dylib` is missing.
- The rebuilt isolated container compiled the Rust test binary and ran the Radarr backup tests successfully. The wider Rust suite reported 59 passed and 5 failures in coordinator-route, avatar-tooling, and quality-profile tests. Its Playwright test passed; the Node suite had multiple existing UI/backfill failures; Python integration tests were skipped by the container.
- `git diff --check`

The local `rawrarr-local` metadata container was rebuilt from this branch and reached healthy status. The loopback app health check passed, the Tailscale Serve route still sent `/` to roarlay and `/direct-api` to the subscriber, and the HTTPS coordinator health check succeeded. The local coordinator rejected the metadata node's registration; its approved-node registry and database are currently owned by another deployment configuration, so they were not changed.

A live Radarr API status check succeeded for version 6.4.4.10685, and Rawrarr issued a native manual backup command. The advertised ZIP path redirected to `/login` despite the API-key header, and Radarr exposes no authenticated ZIP download API in the tested endpoints. Therefore Rawrarr did not store a recovery ciphertext, the restore helper was not run against a real target, and no Radarr container was deployed. Capturing backups from Radarr installations with UI authentication needs a safe authenticated download design; do not place the API key in the download URL.

The focused Python tests use synthetic archives and a local HTTP fixture. The live check uncovered the static-download authentication limitation, so a full live recovery remains unverified.

## Operational notes

The six-hour backup requires an unlocked age vault on a node with the Radarr connection configured. The native archive contains Radarr's database and settings, including third-party integration credentials, but only age ciphertext is placed in `/shared`. The restore helper never creates the Radarr container; the user chooses its host, volume mounts, and deployment method.
