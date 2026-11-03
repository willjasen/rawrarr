# Fix database integration tests

Issue [#130](https://github.com/willjasen/rawrarr/issues/130).

## Objective and initial state

Repair the disposable database integrations that still assume the pre-split standalone app. The role deployment test was blocked by the current pinned signing-root and owner approval requirements; the configuration test did not set an explicit role or coordinator trust. The passkey integration also used node-only environment variables and old coordinator landing-page expectations.

Related context: [regression runner failures](2026-10-26-regression-runner-failures.md) and [implemented browser coverage](2026-10-26-regression-browser-coverage.md).

## Actions

- Updated `test_rawrarr_passkey.py` to boot explicitly as coordinator, use `RAWARR_APPROVED_NODES_FILE` and coordinator WebAuthn settings, assert the public landing page and protected coordinator routes, and confirm malformed registration leaves the database empty.
- Added `--base-url` so the DB integration can exercise a containerized Linux release build while Python/psql run separately. It still refuses DB names that do not end in `_test`.
- Rebuilt the role integration around the supported owner workflow: Chromium creates the first passkey account, creates an age-encrypted signing-root backup, starts metadata and storage subscribers with enrollment enabled, and signs each exact pending request in the coordinator UI. No test-only authentication bypass is used.
- Converted `test_rawrarr_configuration.py` into a test body run from the approved metadata/storage containers, since a standalone launcher cannot exercise node-only routes without coordinator enrollment. It checks group-encrypted config and sidecars, user-wrapped group-key recovery, user/group key separation, HTTPS Radarr validation and save/readback, config redaction, local mirror permissions and production-mode removal, storage-side encrypted sidecars, and encrypted poster delivery. The role test reuses an identity only through captured stdin/stdout pipes; it is not printed or written to the repository.
- The role fixture now uses a uniquely named Docker network and a disposable PostgreSQL `rawrarr_test` database. It starts only its own containers and removes them, the network, synthetic sessions, and temporary files on completion.
- Merging the latest `main` exposed a group-key bootstrap failure for ciphertext stored directly under `/shared`: the helper attempted to change `/` permissions. `encrypt_shared_file` now skips the parent-permission adjustment for filesystem-root parents while retaining it for nested metadata directories. The approved-node test reproduces the non-root container path and verifies group-key creation and recovery.
- Updated the test index with the supported invocations and architecture.
- Built `rawrarr:db-integrations-test` from this worktree and ran the passkey DB integration against an isolated PostgreSQL 17 `rawrarr_test` database in a uniquely named regression network. The coordinator ran from the newly built image; the test exercised its actual PostgreSQL schema and left zero users after malformed registration.
- The first live run corrected an expectation: protected node-only paths return an authorization response on the coordinator, so the boundary assertion now accepts 401 or 404 while requiring no protected data.
- Generated an isolated temporary CA and HTTPS Radarr mock for the role test. The first certificate arrangement passed `curl` but failed Rust TLS verification; replacing it with an explicit CA plus a signed localhost server certificate fixed the integration.

## Verification

- `python3 -m py_compile scripts/tests/test_rawrarr_passkey.py` — passed.
- `git diff --check` — passed.
- `PATH=/tmp/rawrarr-test-bin:$PATH python3 scripts/tests/test_rawrarr_passkey.py --base-url http://localhost:8081 --database-url postgresql://rawrarr:…@localhost:<isolated-port>/rawrarr_test` — passed: first-user challenge, malformed registration, protected-route boundary, DB empty-state assertion, and temporary state cleanup.
- `python3 scripts/tests/test_rawrarr_role_deployment.py --apply --image rawrarr:db-integrations-test` — passed all real owner bootstrap, two signed approvals, PostgreSQL role rows, node-scoped sessions, metadata configuration, storage scan, mirror restart cleanup, role mismatch, outage, and resource cleanup checks.
- `python3 scripts/tests/test_rawrarr_passkey.py --binary /usr/local/bin/rust-arr --database-url postgresql://…/rawrarr_test` — passed first-user challenge, malformed registration rejection, database empty-state assertion, and cleanup in an isolated Linux test container with disposable PostgreSQL.
- `python3 scripts/tests/run_rawrarr_tests_in_docker.py --remove-after` — passed on the merged source: Node 110/110, Playwright 28/28 across 14 browser files, Rust 87/87.
- `python3 -m unittest discover -s scripts/tests -p 'test_*.py'` — 35 passed.
- Standalone Python script checks for album sync, media hashing, deploy targets, local Radarr, Tailscale lock, node provisioning, Radarr recovery, Syncthing setup, storage sharing, and UI bundles — 35 passed.
- `test_rawrarr_syncthing_peers.py` — passed inside an isolated Linux test container using the Syncthing 2.1.5 image binary; the host itself has no Syncthing executable.
- `test_rawrarr_syncthing_orbstack.py` — passed peer connection, folder transfer, ciphertext equality, and cleanup.
- The raw DB password and temporary test port are intentionally omitted here.

## Final state and remaining work

The passkey, role deployment, and configuration database integrations pass with current split-role, signed-root enrollment, and group-age behavior. The full Node, Playwright, Rust, Python unit/script, and Syncthing peer suites pass. All task-owned test containers, networks, generated certificates, identities, DB sessions, and temporary fixtures were cleaned up.
