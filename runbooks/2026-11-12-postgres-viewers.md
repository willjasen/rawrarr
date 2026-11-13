# Private PostgreSQL viewers on do3

Issue: [#451](https://github.com/willjasen/rawrarr/issues/451).

## Scope and decisions

Provide separate Adminer containers for the public and internal coordinator
databases, with database-enforced read-only logins. These are operator tools,
not the sample admin console or subscriber services. Access includes coordinator
table contents; treat query results as private and do not copy them into logs,
issues, or Git. Use only the dedicated viewer login, never application credentials.

The opt-in Compose project is `rawrarr-postgres-viewers`. The public viewer
(`rawrarr-prod-postgres-viewer`) attaches only to `rawrarr-roarlay-prod-control`
and binds host loopback port 7300. The internal viewer
(`rawrarr-int-postgres-viewer`) attaches to `rawrarr-internal-do3-control` and a
dedicated `rawrarr-postgres-viewers-internal-ingress` bridge, and binds loopback
port 7302. Docker suppresses published ports for internal-only networks; the
viewer-only bridge enables loopback publishing without altering the existing
database network. Port 7301 already belongs to public Umami and must
remain untouched. These network names and database container aliases
were read back on do3 on 2026-11-12. No public route, PostgreSQL port, application
secret mount, subscriber data mount, or persistent Adminer volume is introduced.
Existing peers on each control network can also reach that viewer; database
login remains required. Adminer accepts manually entered server/login values,
so network isolation and use of the dedicated login remain important.

## Adoption and compatibility

Deploy only after this issue's changes are published on GitHub `main`, using an
authenticated archive of the exact published commit in a new host-local source
directory. Do not deploy working-tree files. Adminer is third-party support
software (`adminer:5.4.1-standalone`), not a Rawrarr runtime image rebuild.
There is no application schema, backend, subscriber, or protocol migration.

`deploy/do3/postgres-viewer-v1.sql` is a versioned **administrative opt-in**
migration, applied separately to each existing coordinator database. It does not
belong to application SQLx migrations: public user deployments need not create
operator accounts. It creates `rawrarr_viewer`, grants SELECT on existing public
tables, and sets default SELECT grants for future tables owned by `rawrarr`.
Mixed application versions keep working. Reruns preserve the password and fail
on unmanaged roles, privileged attributes, role memberships, object ownership,
or executable SECURITY DEFINER functions. A failure rolls back all SQL changes.
Future schema/owner changes or SECURITY DEFINER functions require another access
review; do not bypass the guard to enable the viewer.

Before adopting, verify no application-table write/CREATE privileges have been
granted to PUBLIC or through another schema. This setup assumes the existing
coordinator's default PostgreSQL 17 public-schema permissions and rawrarr-owned
application tables. Custom deployments require a separate grants review.

## Deployment

Connect through verified Tailscale SSH as root and run host commands in tmux
session `ai`. Confirm `docker context show` is `default`, check free storage,
verify both database containers are healthy and have the networks above, and
check ports 7300/7302 are unused. Do not restart the databases, applications,
Tailscale, or any unrelated service.

Atomically claim a distinct deployment marker for each new viewer before
changing it, using the established per-container lock directory on do3. Hold
that viewer's marker through SQL provisioning and browser/health verification;
never remove another task's marker. Apply one environment at a time.

From the published source directory, apply the migration without echoing any
database password:

```sh
docker exec -i rawrarr-prod-postgres psql -X -U rawrarr -d rawrarr \
  < deploy/do3/postgres-viewer-v1.sql
docker exec -i rawrarr-int-postgres psql -X -U rawrarr -d rawrarr \
  < deploy/do3/postgres-viewer-v1.sql
```

For each database, interactively set a different generated password with psql's
hidden password prompt. Save it in the operator's password manager; do not put it
in shell arguments/history, Compose environment, a runbook, or Git:

```sh
docker exec -it rawrarr-prod-postgres psql -X -U rawrarr -d rawrarr \
  -c '\password rawrarr_viewer'
docker exec -it rawrarr-int-postgres psql -X -U rawrarr -d rawrarr \
  -c '\password rawrarr_viewer'
```

Pull and inspect the Adminer image once, then recreate only the two new viewers
using that local image. Record its digest and architecture with deployment
evidence, without credentials:

```sh
docker pull adminer:5.4.1-standalone
docker image inspect adminer:5.4.1-standalone \
  --format '{{.Os}}/{{.Architecture}} {{json .RepoDigests}}'
docker compose -p rawrarr-postgres-viewers \
  -f deploy/do3/docker-compose.postgres-viewers.yml config --quiet
docker compose -p rawrarr-postgres-viewers \
  -f deploy/do3/docker-compose.postgres-viewers.yml \
  up -d --no-deps --pull never public-viewer
docker compose -p rawrarr-postgres-viewers \
  -f deploy/do3/docker-compose.postgres-viewers.yml \
  up -d --no-deps --pull never internal-viewer
curl -fsS http://127.0.0.1:7300/ > /dev/null
curl -fsS http://127.0.0.1:7302/ > /dev/null
```

Confirm actual container network membership and loopback-only published bindings.
Verify both database/app health checks remain healthy; verify the internal HTTPS
`https://rawrarr.risk-mermaid.ts.net/health` route still identifies roarlay.
No Serve handler is changed. Release only this task's viewer locks after
verification completes or the corresponding deployment has stopped.

## Use

On the operator's Mac, keep this SSH tunnel open:

```sh
ssh -N -o ExitOnForwardFailure=yes \
  -L 127.0.0.1:7300:127.0.0.1:7300 \
  -L 127.0.0.1:7302:127.0.0.1:7302 root@do3.risk-mermaid.ts.net
```

Keep host-key verification enabled; stop on an unknown or changed key rather
than bypassing it. Use the verified local SSH key/host entry.

Open `http://127.0.0.1:7300/` for public or `http://127.0.0.1:7302/` for internal.
Choose **PostgreSQL**, retain the default server (`rawrarr-prod-postgres` or
`rawrarr-int-postgres`), username `rawrarr_viewer`, database `rawrarr`, and the
corresponding viewer password. Browse tables or use SQL for SELECT queries.
Encrypted values remain encrypted; the viewers receive no age identities.

Confirm authenticated browsing succeeds and a write is rejected by database
permissions even after `SET default_transaction_read_only = off`. Use a
transaction containing `UPDATE public._sqlx_migrations SET success = success
WHERE false` followed by ROLLBACK; no row is modified even if permissions are
unexpectedly wrong. Never run a destructive verification against live data.

## Verification and status

- The published Adminer tag exists; Compose configuration validates locally.
- Focused test: `python3 scripts/tests/test_postgres_viewers.py`.
- Disposable database integration passed (all four focused tests):
  `RAWARR_TEST_POSTGRES_VIEWER=1 python3 scripts/tests/test_postgres_viewers.py`.
  Requires Docker and an existing native `postgres:17-alpine` image. See the
  [test index](../scripts/tests/README.md); it never uses live databases.
- Deployment, password setup, and live login remain pending until
  the change is published on main and the operator completes password setup.
- Initial deployment of `d4e47521ad9d08338bdbc38cacc7f026ec231605`
  succeeded for public, including authenticated SELECT and permission-denied
  write verification. Internal exposed Docker's internal-only port suppression;
  the follow-up adds a viewer-only ingress bridge. Both new passwords are
  generated into each environment's protected `secrets/postgres-viewer-password`
  file (root-owned, mode 0600), not application password files or Git.
- Adminer's upstream UI is not customized here. Desktop/mobile Playwright
  coverage for it is not added; live upstream layout/login is a verification gap.
  No Rawrarr browser-facing UI is changed.

## Recovery and rollback limits

On SQL failure, stop that environment's rollout and review the reported cause;
do not recreate its existing application/database or loosen grants as a fallback.
A password setup failure leaves the new account unable to log in; repeat only
the hidden password prompt. A viewer startup failure does not affect the app.

To disable a viewer, claim its deployment lock and stop only that Compose
service. Disable `rawrarr_viewer` with `ALTER ROLE rawrarr_viewer NOLOGIN` in that
database and terminate its existing sessions if immediate revocation is needed.
Do not run `compose down` on either existing application project or remove any
database volume. The harmless grants can remain disabled; full role removal
requires reversing default privileges and grants with `DROP OWNED BY
rawrarr_viewer` in this database before dropping the role. Review ownership first
and perform removal only with explicit approval. Re-adoption after NOLOGIN
requires a deliberate `ALTER ROLE rawrarr_viewer LOGIN`.
