# Store compressed Rawrarr logs in shared app data

## Objective and initial state

Write compressed Rawrarr operational logs under the existing Syncthing-shared `/shared` folder. The application previously printed startup and scanner status only to stdout. The shared data-domain registry already separates `/shared` from local `/data` and storage-node `/media`.

## Actions

- Added a safe Rust logger that writes gzip-compressed JSONL members beneath `$RAWARR_SHARED_DATA_DIR/logs/<device-id>/`. Files rotate at 8 MiB per part. Startup and completed library scans are recorded.
- Restricted records to fixed event names, timestamps, device IDs, and numeric scan summaries. Request bodies, URLs, credentials, media paths, and other user-provided text are not accepted by the logger API.
- Added unit coverage for concatenated gzip records, allowlisted fields, and rejection of path-like device IDs. Rust continues to forbid unsafe code.
- Updated the data-domain documentation with `/shared/logs` and clarified that Tailscale carries Rawrarr application/control traffic only, never media files or streams. Syncthing remains the shared-folder transport.
- Recorded the Tailscale transport boundary from the user's clarification without changing the earlier Tailscale runbook; the active plan remains linked from [the prior runbook](2026-10-18-rawrarr-tailscale-integration-plan.md).

## Verification

Rust 1.85 formatting passed; locked tests passed (8), including concatenated gzip decoding, field allowlisting, and path rejection. The locked release build and OrbStack image build passed. The updated Rawrarr container is running on `127.0.0.1:8084`; `/health` returned HTTP 200 `ok`, and PostgreSQL remained healthy. Two valid gzip log files with two startup records were found under the Finder-visible `shared-data/logs/<device-id>/` mount. Decompression and JSON parsing succeeded, and all fields matched the allowlist. No library scan was run. The live Rawrarr bind mount was not shown to be connected to the separate temporary Syncthing test folders, so peer replication of these specific log files remains unverified.

## Final state and remaining work

The implementation is active in the local OrbStack instance. Log retention and whether shared logs should also be age-encrypted remain future policy decisions; current records are compressed and contain only the documented allowlist.
