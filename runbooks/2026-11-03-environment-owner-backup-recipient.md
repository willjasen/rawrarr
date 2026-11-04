# Environment owner recipient for internal coordinator backups

## Scope and tracking

Continuation of [issue #299](https://github.com/willjasen/rawrarr/issues/299) and
[PR #300](https://github.com/willjasen/rawrarr/pull/300). Replace manual backup-key
provisioning with an explicit authenticated owner action and a PostgreSQL pin.
This implementation does not deploy, register a live owner, execute a production
backup, or modify either internal or public running services.

## Trust and data boundaries

`rawrarr_environment_owner` is a singleton containing the existing authenticated
account UUID, personal age public recipient, original source fossil ID, and
creation timestamp. It is not the ML-DSA instance-signing-root table and contains
no private identity. The source ID is historical provenance, not a cascading
foreign key: removing a fossil must not erase the recipient needed for recovery.
Deleting the owner account requires a separate explicit ownership/data decision;
the foreign key deliberately prevents silently deleting the owner pin.

Only a session authenticated to the coordinator's own origin can provision.
The request supplies one fossil ID, not a recipient or URL. The coordinator
checks that it is an account-owned metadata node, then uses its configured relay
route and an existing delegated node session to fetch
`GET /api/metadata/export/owner-recipient`. Redirects are disabled, the response
is bounded to 4096 bytes, and the public post-quantum recipient must pass an actual
age encryption validation before pinning. Other fossil data is not fetched.

The fossil derives the public value from its unlocked vault's user identity or
an existing protected node-local `/data/age-identity`. A locked fossil with no
remembered identity returns 423; unsafe/unreadable remembered state is an error.
Reading does not generate an identity or initialize/migrate group configuration.
The endpoint is metadata-only and session-protected. It does not expose a private
key, group identity, or node identity, and is not added to the general coordinator
relay allowlist.

The committing transaction rechecks fossil ownership and serializes owner
claims. Initial provisioning requires exactly one account and must agree with
any established signing-root owner association, without reusing signing-key
material. Same-owner/same-recipient retries are idempotent and preserve original
provenance. Conflicts never replace the pin. The authenticated fossil/account
relationship is the trust source; this is not remote attestation or proof that
the owner still has an independent recovery copy.

## Adoption prerequisites

After separate deployment authorization, publish and merge the release first.
Deploy the exact published `main` revision using the established host-local image
and deployment-lock procedure. Upgrade the internal do3 coordinator and at least
one relevant Mac fossil. Do not deploy to or query do3's public coordinator as a
substitute. The coordinator's embedded SQLx migrator applies additive migration
`0012_environment_owner.sql`; existing deployments start with an empty owner table.
No synchronized-file migration or subscriber database is required.

An upgraded fossil can still operate with an older coordinator because the
heartbeat and existing unlock protocols do not change. An old fossil returns
404 for this new endpoint, which the upgraded coordinator reports as 428 with
instructions to upgrade. There is no fallback to older group-recipient exports.

## Authenticated provisioning and readback

Sign in with the existing passkey at the internal coordinator
`https://rawrarr.risk-mermaid.ts.net/`. Select a currently online owned fossil ID
from the existing Devices view. Ensure it has a configured coordinator relay
route. If necessary, unlock that fossil with the existing owner identity using
the normal encrypted unlock flow; do not generate or paste a replacement key
into any coordinator request.

From that signed-in coordinator page's browser developer console, deliberately
run the following, replacing only the fossil ID:

```js
const provision = await fetch('/api/workspace/environment-owner', {
  method: 'POST',
  credentials: 'same-origin',
  headers: {'Content-Type': 'application/json'},
  body: JSON.stringify({nodeId: 'YOUR-EXISTING-FOSSIL-ID'})
});
if (!provision.ok) throw new Error(await provision.text());
const owner = await provision.json();
```

The browser supplies the same-origin `Origin` header automatically. This is a
documented API action, not a new settings page. Read back the stored public
record with:

```js
const readback = await fetch('/api/workspace/environment-owner', {
  credentials: 'same-origin',
  cache: 'no-store'
});
if (!readback.ok) throw new Error(await readback.text());
const pinnedOwner = await readback.json();
```

The returned `ownerUserId`, `ageRecipient`, and `sourceNodeId` are public
provisioning metadata scoped to the signed-in owner. Deliberately compare the
public recipient to the existing owner identity on an owner-controlled device;
never print the private identity, copy credentials into commands or runbooks, or
store session cookies in repository files. A syntactically valid public recipient
is not sufficient disaster-recovery evidence: verify decryption of a separately
authorized backup on a device with the matching private identity.

## Backup behavior and failure recovery

Both new backups and `--encrypt-existing` resolve the recipient from the pinned
record using a fixed read-only `psql` query in the internal PostgreSQL container,
then run the existing age preflight before collecting plaintext. Fossils can be
offline afterward. Missing schema, unprovisioned ownership, malformed/ambiguous
results, database failure, and age validation failure abort without fallback.
No `backup-age-recipient` file is read and `--recipient-file` is rejected.
Existing obsolete recipient files need not be deleted; they no longer influence
the script. Do not remove any old backup or runtime file as part of this change.

Provisioning errors distinguish unauthenticated/wrong-origin requests (401/403),
unowned or non-fossil nodes (403), unavailable configured route (503), offline or
invalid fossil responses (502), old fossil versions (428), locked fossils (423),
and conflicting/ambiguous ownership (409). Database failures return 503. Resolve
the stated prerequisite and retry the same action. Failed transactions leave no
partial owner record; concurrent claims cannot silently replace one another.

On a conflict, preserve the pin and existing backups. Do not edit the database
recipient or generate another identity to make the error disappear. Rotation and
ownership transfer are out of scope and need a separate authenticated recovery
design that retains access to old ciphertext.

The database dump includes the owner record. Restoring it retains the public pin
but does not restore the private recovery key. Rollback may run older application
code while leaving this additive table intact; do not drop the table. Older
backup scripts do not automatically adopt this database source and must not be
used as an implicit manual-file fallback.

## Focused verification

Use synthetic identities and a disposable database ending in `_test`. In a
native isolated test clone prepared from the current source with
`scripts/tests/prepare-test-clone.sh`, disconnect bridge networking before tests.
A disposable PostgreSQL 17 sidecar can share the clone's network namespace;
publish no host ports and connect through loopback.

```sh
python3 scripts/tests/test_internal_coordinator_backup.py
cargo test --locked --offline --manifest-path rust-arr/Cargo.toml owner
RAWARR_TEST_DATABASE_URL=postgres://postgres@127.0.0.1/rawrarr_owner_test \
  cargo test --locked --offline --manifest-path rust-arr/Cargo.toml \
  environment_owner_database_and_api -- --ignored
cargo test --locked --offline --manifest-path rust-arr/Cargo.toml dashboard_event_tests
cargo check --locked --offline --manifest-path rust-arr/Cargo.toml
```

The ignored integration test is explicitly invoked in the last command, not
silently treated as covered by the unit-test filter. It creates a unique
disposable schema, exercises the migration, authenticated handlers and a local
fossil HTTP fixture, and removes that schema on success.

The Python suite's actual PostgreSQL resolver case is opt-in. Set
`RAWARR_TEST_BACKUP_POSTGRES_CONTAINER` to a disposable PostgreSQL 17 container
whose name starts with `rawrarr-test-regression-owner-postgres-`, with a database
named `rawrarr_owner_test` and the test `postgres` role. It redirects only the
resolver's container/database connection into that fixture; it does not contact
the internal deployment. Without the variable this case is reported as skipped.

## Execution record

Completed the focused checks in a uniquely named native AMD64 linked clone of
`rawrarr-testing-base:local`, using the current working source and lockfile.
Prepared the clone with network access, then disconnected bridge networking
before tests. PostgreSQL 17 ran as a disposable sidecar sharing only the clone's
network namespace, with no host-published ports.

- `RAWARR_TEST_BACKUP_POSTGRES_CONTAINER=rawrarr-test-regression-owner-postgres-e3d849d9 python3 -B scripts/tests/test_internal_coordinator_backup.py`: 13 passed, including the actual read-only `psql` output and offline-source case.
- `cargo test --quiet --locked --offline --manifest-path /workspace/rust-arr/Cargo.toml owner`: 6 passed; the database test was explicitly invoked separately.
- `RAWARR_TEST_DATABASE_URL=postgres://postgres@127.0.0.1/rawrarr_owner_test cargo test --quiet --locked --offline --manifest-path /workspace/rust-arr/Cargo.toml environment_owner_database_and_api -- --ignored`: 1 passed.
- `cargo test --quiet --locked --offline --manifest-path /workspace/rust-arr/Cargo.toml dashboard_event_tests`: 2 passed, covering the reused dashboard fixture and non-fossil role rejection.
- `cargo check --quiet --locked --offline --manifest-path /workspace/rust-arr/Cargo.toml`: passed.
- `git diff --check`: passed.

The host formatter could not start because its Homebrew LLVM dependency referred
to a missing Z3 library. Installed the matching rustfmt component only inside the
disposable clone while it had network access, disconnected it again, and
formatted the new Rust files there. No shared toolchain or base image was changed.

No live environment, production backup, owner-device recovery, full multi-node
environment suite, or browser UI was exercised. The fossil's real public response
handler and remembered-user derivation were tested locally; coordinator fetching
used a controlled HTTP fossil fixture with a real delegated PostgreSQL session.
An authorized internal rollout and owner-device decryption check remain required
before claiming operational recovery readiness.

## Base-branch merge verification

Merged the freshly fetched `origin/main` at `698b9fd` into
`codex/internal-coordinator-backup` after confirming the local and remote task
branches were identical at `f96597d`. This incorporates 195 upstream commits
without rebasing or rewriting branch history. The two content conflicts were in
`scripts/README.md` and `scripts/tests/README.md`; retained the upstream classifier
description and all upstream test entries alongside the backup/owner entries.
Rust source merged automatically, and migration `0012` remains distinct from
upstream migration `0011`.

Verified the merged source in a new native AMD64 linked test clone, prepared with
the current lockfile and then disconnected from bridge networking. A disposable
PostgreSQL 17 sidecar shared only that clone's network namespace, without host
ports. Re-ran the owner filter (6 passed), explicitly invoked database migration/
API integration (1 passed), dashboard filter (2 passed), and locked offline
`cargo check` (passed). Re-ran the Python backup suite with its real PostgreSQL
resolver fixture enabled (13 passed). `git diff --check` and the staged equivalent
passed, with no unresolved conflict paths. Full suites, deployment, and production
backup execution were not performed. This merge verification does not establish
that historical host paths/container names in the original backup procedure
match a current deployment.
