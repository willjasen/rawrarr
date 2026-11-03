# Add a three-day trial for public Rawrarr accounts

## Objective

Record the agreed three-day trial for new users on public deployments, with no trial for internal deployments. Track this implementation under [Stripe billing issue #160](https://github.com/willjasen/rawrarr/issues/160).

## Implementation

- Added nullable `trial_started_at` to `rawrarr_users`. New accounts record the database time only when `RAWARR_BILLING_AUDIENCE=public`; legacy accounts remain unset.
- Exposed the UTC trial start and end through the authenticated workspace profile only on public deployments. The account Settings profile displays the end in the browser's local time and indicates when it has passed.
- Configured the public coordinator Compose definition to default the audience to `public`. The standard local and internal Mac mini Compose definitions explicitly use `internal`.
- Updated subscription and product direction docs to describe the three-day duration and current reporting-only behavior.

## Limits

Trial expiry does not yet gate product features or connect to Stripe entitlements. The timestamp currently begins when passkey registration completes. Issue #160 still tracks checkout, webhook processing, subscription lifecycle, and Stripe test-mode validation.

## Verification

`test_rawrarr_trial.mjs` checks public-only timestamp creation, the three-day expiry, legacy-account behavior, Compose audience settings, authenticated profile visibility, and local-time display. The focused trial and coordinator profile tests passed locally (32 assertions). The OrbStack `rawrarr-test-regression` run passed all 118 Node tests and all 97 Rust tests. Its Chromium suite had 11 failures in existing movie artwork timing, movie-link reachability, stale library snapshot, navigation, shared-library, and Sonarr settings cases; no browser test covers trial behavior. Python integration tests were skipped by the runner. No PostgreSQL-backed migration/account-creation integration test was run, and the public and internal deployments were not rebuilt or restarted.
