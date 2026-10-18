# Use lowercase Rawrarr Application Support directory

## Objective and initial state

Use `~/Library/Application Support/rawrarr` as the canonical local Rawrarr data root. The persistent Compose service mounted `~/Library/Application Support/Rawrarr/shared-data`; because the host filesystem is case-insensitive, the lowercase and uppercase paths resolved to the same directory and inode. The directory contains shared encrypted config, logs, metadata folders, recovery data, and the PostgreSQL password file.

## Actions

- Updated the Compose default shared-data path and current user-facing data-path documentation to lowercase `rawrarr`.
- Renamed the directory through a temporary sibling name to force the case-only rename on the case-insensitive volume; no files were copied or deleted.
- Recreated only the persistent app container so `/shared` binds to the lowercase canonical path. The PostgreSQL volume and host shared-data contents were preserved.

## Verification

- Compose resolves the app's `/shared` bind to `/Users/willjasen/Library/Application Support/rawrarr/shared-data`.
- `config.aged` and `orbstack-postgres-password` remain present under the lowercase root; Compose configuration validates with the password-file path under that root.
- The local app returns `ok` from `http://127.0.0.1:7297/health` and Docker reports the container healthy.
- The macOS volume is case-insensitive, so looking up the old uppercase spelling still resolves to the same directory. The stored/canonical name and Compose source are lowercase.

## Final state and remaining work

The canonical local Rawrarr directory is lowercase. No files were copied, deleted, or re-encrypted. The app is running on port 7297 with the existing database volume and shared files intact.
