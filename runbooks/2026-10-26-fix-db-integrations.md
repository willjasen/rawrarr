# Fix database integration tests

Issue [#130](https://github.com/willjasen/rawrarr/issues/130).

## Objective and initial state

Repair the disposable database integrations that still assume the pre-split standalone app. The role deployment test was blocked by the current pinned signing-root and owner approval requirements; the configuration test did not set an explicit role or coordinator trust. The passkey integration also used node-only environment variables and old coordinator landing-page expectations.

Related context: [regression runner failures](2026-10-26-regression-runner-failures.md) and [implemented browser coverage](2026-10-26-regression-browser-coverage.md).

## Actions

- Updated `test_rawrarr_passkey.py` to boot explicitly as coordinator, use `RAWARR_APPROVED_NODES_FILE` and coordinator WebAuthn settings, assert the public landing page and protected coordinator routes, and confirm malformed registration leaves the database empty.
- Added `--base-url` so the DB integration can exercise a containerized Linux release build while Python/psql run separately. It still refuses DB names that do not end in `_test`.
- Updated the test index with current prerequisites and documented that the standalone configuration integration still requires an owner-approved metadata node. Rust configuration, schema, and Arr settings tests cover its non-DB vault behavior in the meantime.
- Built `rawrarr:db-integrations-test` from this worktree and ran the passkey DB integration against an isolated PostgreSQL 17 `rawrarr_test` database in a uniquely named regression network. The coordinator ran from the newly built image; the test exercised its actual PostgreSQL schema and left zero users after malformed registration.
- The first live run corrected an expectation: protected node-only paths return an authorization response on the coordinator, so the boundary assertion now accepts 401 or 404 while requiring no protected data.
- The role deployment integration still needs the current browser-driven encrypted-root enrollment and approval flow before it can start metadata/storage nodes. No node-auth bypass was added.

## Verification

- `python3 -m py_compile scripts/tests/test_rawrarr_passkey.py` — passed.
- `git diff --check` — passed.
- `PATH=/tmp/rawrarr-test-bin:$PATH python3 scripts/tests/test_rawrarr_passkey.py --base-url http://localhost:8081 --database-url postgresql://rawrarr:…@localhost:<isolated-port>/rawrarr_test` — passed: first-user challenge, malformed registration, protected-route boundary, DB empty-state assertion, and temporary state cleanup.
- The raw DB password and temporary test port are intentionally omitted here.

## Final state and remaining work

Passkey DB integration is repaired and verified. Full role deployment and configuration API integrations remain pending because they need a disposable owner-approved metadata node. The temporary regression containers/network and test-only registry fixture are removed after verification.
