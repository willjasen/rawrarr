# Move local Rawrarr shared data outside Codebox

## Objective and initial state

Place the local Compose `/shared` bind mount outside Codebox while keeping the files persistent and directly browseable in Finder. The running `rawrarr-local` app used `/Users/willjasen/.binds/sync/Codebox/arr-services/shared-data`. The destination `/Users/willjasen/Library/Application Support/Rawrarr/shared-data` did not exist; the source contained an age recipient and compressed operational logs. File names and sizes were inspected without reading their contents.

## Changes

- Change the local Compose default to `$HOME/Library/Application Support/Rawrarr/shared-data`, with `RAWARR_SHARED_DATA_PATH` still available for an explicit override.
- Copy and verify the existing shared-data tree at the new path. Stop the app before final synchronization to avoid concurrent writes, then restart it with the new bind mount.
- Retain the old Codebox copy as a rollback snapshot; Compose no longer uses it.
- Document the default path and Finder access.

## Verification

Compose resolved `/shared` to `/Users/willjasen/Library/Application Support/Rawrarr/shared-data`. The old app container was stopped before a final `rsync`; `diff -qr` found no differences at cutover. The rebuilt `rawrarr-local` app is running with `/shared` mounted to the Application Support path, and `/health` returns `ok`. The new folder is open in Finder. The old Codebox copy remains untouched as a rollback snapshot; it is no longer mounted by Compose.
