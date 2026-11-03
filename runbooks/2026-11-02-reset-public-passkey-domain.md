# Reset public coordinator passkeys for rawrarr.net

Issue: [#341](https://github.com/willjasen/rawrarr/issues/341)

## Scope

The public Rawrarr app remains at `https://roarlay.rawrarr.net/`. Configure its WebAuthn origin as `https://roarlay.rawrarr.net` and RP ID as `rawrarr.net`, allowing the app origin to use credentials scoped to the apex domain. The `rawrarr.net` apex remains the Netlify landing page.

The current deployed coordinator is misconfigured with origin and RP ID `rawrarr.willjasen.net`. That legacy host returns 404. Will selected resetting the existing coordinator account state to start fresh with a new passkey. The verified database currently contains one user, one passkey, zero approved nodes, one pending enrollment, zero instance signing roots, zero payments, and zero subscriptions. Resetting will remove the old account/passkey and pending enrollment.

## Compatibility and reset plan

There is no schema or subscriber protocol change. A fresh coordinator PostgreSQL volume will run the same embedded SQLx migrations from the published Rawrarr image. Before cutover, make and verify a custom-format dump of the existing coordinator database. Leave both the dump and current named PostgreSQL volume intact; use a new, uniquely named named volume for first-user setup rather than deleting or reinitializing the existing volume. Rollback can reattach the old volume and restore the prior container environment, but the old credential remains unusable on the current hostname until its legacy origin is restored.

Keep the separate `rawrarr-stripe` database/service, Cloudflare Tunnel, `roarlay.rawrarr.net`, `/shared`, subscriber `/data`, identities, credentials, and local media untouched. The pending enrollment is the only non-account row confirmed for the reset; the new owner will need to re-enroll any nodes afterward.

## Adoption and verification

The tracked public Compose defaults use `RAWARR_COORDINATOR_WEB_RP_ID=rawrarr.net` and `RAWARR_COORDINATOR_WEB_ORIGIN=https://roarlay.rawrarr.net`. The public host has explicit legacy environment overrides, so deployment must update those values in the host's existing public coordinator configuration after this change is published. Build/deploy only the exact published GitHub `main` revision using the existing do3 Compose project and per-container deployment lock. Start Roarlay against the new volume and verify health, the `/auth` route, owner passkey registration/sign-in, and continued TLS health at the app hostname. Do not direct `rawrarr.net` to the app; it remains the static landing page.

`node --test scripts/tests/test_rawrarr_coordinator_passkey.mjs` verifies the public RP ID/origin pair; the Rust WebAuthn builder test verifies that `rawrarr.net` is accepted for `https://roarlay.rawrarr.net`. Record deployment results and the verified backup path here after cutover.
