# Isolate the two-container Rawrarr test group

Give the OrbStack Syncthing web/storage integration test a discoverable Docker project identity named `rawrarr-testing`. Keep it separate from the user's persistent `rawrarr-local` application and PostgreSQL project. Related procedure: [Rawrarr Syncthing peer sync](2026-10-17-rawrarr-syncthing-peer-sync.md).

## Initial state

- `rawrarr-local-rawrarr-1` and `rawrarr-local-postgres-1` were running in OrbStack under Compose project `rawrarr-local`; both were healthy.
- The integration script created random `rawrarr-web-test-*` and `rawrarr-storage-test-*` containers without Docker project labels, so the pair was not grouped separately in OrbStack.
- Existing exited `rawrarr-web-test-7c990cb9` and `rawrarr-storage-test-7c990cb9` containers were left untouched.

## Changes and verification

- Updated `scripts/tests/test_rawrarr_syncthing_orbstack.py` to name test containers and networks with a `rawrarr-testing` prefix and add Docker project/service labels. The integration test now checks these labels before peer setup.
- Updated the test index to document the group and the `--keep` option.
- Ran `python3 scripts/tests/test_rawrarr_syncthing_orbstack.py --keep`. It passed the two-peer connection, folder-role, nested-file SHA-256, and byte-identical age-encrypted config checks.
- Left the verified test group running in OrbStack: `rawrarr-testing-web-08d1f47d`, `rawrarr-testing-storage-08d1f47d`, network `rawrarr-testing-net-08d1f47d`. Temporary fixture data is at `/var/folders/l_/_fxqjgld71ngt2t2wvqqwq700000gn/T/rawrarr-orbstack-sync-tut5tzb5`.
- Syncthing GUIs are available at `http://127.0.0.1:57341` (web) and `http://127.0.0.1:57342` (storage).

The default test mode still removes its temporary containers, network, and files. Use `--keep` to inspect them after a passing run. Remove the test resources when finished with `docker --context orbstack rm -f rawrarr-testing-web-08d1f47d rawrarr-testing-storage-08d1f47d` and `docker --context orbstack network rm rawrarr-testing-net-08d1f47d`; then remove the printed temporary test-data directory if desired. These commands do not target `rawrarr-local`.

## Final state

`docker compose -p rawrarr-testing ps --all` discovers both running containers under the intended project. `docker inspect` confirms the web/storage service labels and confirms both `rawrarr-local` containers remain healthy under their original project. The local app health endpoint at `http://127.0.0.1:8084/health` returned `ok`. Python bytecode compilation and `git diff --check` passed; generated `__pycache__` was removed. Task-owned changes were staged after reviewing the staged diff; pre-existing changes to `docs/node-types.md` and the container-distribution runbook remain unstaged.
