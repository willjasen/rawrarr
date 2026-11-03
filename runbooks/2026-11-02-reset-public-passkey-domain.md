# Reset public coordinator passkeys for rawrarr.net

Issue: [#341](https://github.com/willjasen/rawrarr/issues/341)

## Scope

The public Rawrarr app remains at `https://roarlay.rawrarr.net/`. Configure its WebAuthn origin as `https://roarlay.rawrarr.net` and RP ID as `rawrarr.net`, allowing the app origin to use credentials scoped to the apex domain. The `rawrarr.net` apex remains the Netlify landing page.

The current deployed coordinator is misconfigured with origin and RP ID `rawrarr.willjasen.net`. That legacy host returns 404. Will selected resetting the existing coordinator account state to start fresh with a new passkey. The verified database currently contains one user, one passkey, zero approved nodes, one pending enrollment, zero instance signing roots, zero payments, and zero subscriptions. Resetting will remove the old account/passkey and pending enrollment.

## Compatibility and reset plan

There is no schema or subscriber protocol change. Before cutover, make and verify a custom-format dump of the existing coordinator database. Keep the existing PostgreSQL container and named volume intact, and create a separate, uniquely named empty database in that PostgreSQL instance for first-user setup. The published image applies the same embedded SQLx migrations to the new database at startup. Leave the old database, its volume, and the verified dump intact. Rollback can point Roarlay back to the old database and prior image/configuration; the old credential remains unusable on the current hostname until its legacy origin is restored.

Keep the separate `rawrarr-stripe` database/service, Cloudflare Tunnel, `roarlay.rawrarr.net`, `/shared`, subscriber `/data`, identities, credentials, and local media untouched. The pending enrollment is the only non-account row confirmed for the reset; the new owner will need to re-enroll any nodes afterward.

## Adoption and verification

The tracked public Compose defaults use `RAWARR_COORDINATOR_WEB_RP_ID=rawrarr.net` and `RAWARR_COORDINATOR_WEB_ORIGIN=https://roarlay.rawrarr.net`. The public host has explicit legacy environment overrides, so deployment must update those values in a host-local public coordinator override after this change is published. Build/deploy only the exact published GitHub `main` revision using the existing do3 Compose project and per-container deployment lock. Point Roarlay at the new database and verify health, the `/auth` route, first-user setup availability, and continued TLS health at the app hostname. Do not direct `rawrarr.net` to the app; it remains the static landing page.

`node --test scripts/tests/test_rawrarr_coordinator_passkey.mjs` verifies the public RP ID/origin pair; the Rust WebAuthn builder test verifies that `rawrarr.net` is accepted for `https://roarlay.rawrarr.net`. Record deployment results and the verified backup path here after cutover.

## Deployment record

- PR [#342](https://github.com/willjasen/rawrarr/pull/342) merged to `main` as `e2bd850a8b6464702c26b58515bccc5da1f7efce`. On do3, fetched the authenticated GitHub archive directly at that revision; SHA-256: `da1bf6960b7337742493b63db589dd9f7b277bde6c3d5ad206cc4dba905160f1`.
- Built `rawrarr:e2bd850-amd64` natively on do3 and verified `linux/amd64` plus the full revision label. Preserved the prior image as `rawrarr:rollback-pre-passkey-e2bd850-amd64`; advanced `rawrarr:internal-base-amd64` only after the new build succeeded.
- Verified the existing coordinator database had one user, one passkey, zero nodes, and one pending enrollment (and zero signing roots, payments, or subscriptions). Created `/opt/rawrarr-roarlay-prod/backups/pre-passkey-reset-e2bd850-20261103.dump` (mode 0600, 53,765 bytes); `pg_restore --list` succeeded inside the PostgreSQL container. SHA-256: `d4d3f2bad439a2f1773b1514fba7011fd14d1f6c8927e733c585fed251cca152`.
- Created `rawrarr_passkey_reset_20261103` inside the existing PostgreSQL instance. Roarlay now uses that database; the existing PostgreSQL container and named volume were not restarted, replaced, or removed. The old database still contains the original account and pending enrollment.
- Recreated only public Roarlay under `/opt/rawrarr-roarlay-prod/.container-rebuild-roarlay.lock`, then released and verified the lock. The new database reached SQLx migration 11 and contains zero users, passkeys, nodes, or enrollments. The app reports healthy with zero restarts and the expected image revision. Local and public `/health` and `/auth` checks returned HTTP 200; public `/api/auth/status` returned `{"userExists":false}`.
- The owner must still register and sign in with a new passkey at `https://roarlay.rawrarr.net/auth`; the previous account/passkey remains preserved in the old database and dump. No subscriber nodes needed re-enrollment because none were approved in the old coordinator database.
