# Deploy the Stripe webhook service on do3

## Objective

Deploy the published Stripe webhook service beside Roarlay on `rawrarr-roarlay`, using the same hosted account database that stores passkey-linked users. Keep the webhook service separate from subscriber nodes and restrict database privileges to billing tables and `rawrarr_users.id`.

## Implementation and safety

- The webhook service maintains its own `rawrarr_stripe_migrations` ledger because Roarlay already uses `_sqlx_migrations` in the shared account database.
- The do3 Compose overlay joins only `rawrarr-roarlay-prod-control`; it publishes no host port. Caddy forwards only `/v1/webhooks/stripe` to the service.
- The service reads the DB password and Stripe signing secret from protected mounted files. An empty signing secret leaves the service unready and returns 503 for webhook requests.
- Build and deploy only the exact published `main` commit. Preserve Roarlay, PostgreSQL, Headscale, Tailscale, and current Caddy routes; validate Caddy before reloading it.

## Deployment prerequisites

- Create the restricted PostgreSQL login `rawrarr_stripe`, granting database CONNECT, public-schema USAGE/CREATE, and only SELECT/REFERENCES on `rawrarr_users(id)`. Do not grant access to `rawrarr_passkeys`.
- Store the login password and Stripe webhook signing secret under `/opt/rawrarr-roarlay-prod/secrets/` with owner UID/GID 10001 and mode 0400 so the non-root container can read them.
- Add the route from `deploy/do3/stripe-service.Caddyfile` inside the `rawrarr.willjasen.net` site block before its catch-all handler.
- Create the webhook endpoint in Stripe test mode after the route is live, select supported events, and install that endpoint's signing secret in the host secret file before expecting healthy status or accepting events.

## Verification

- In Mac mini OrbStack, `cargo test --offline --locked` passed (7 Rust tests), debug and release `cargo build --offline --locked` passed, and `node --test scripts/tests/test_rawrarr_stripe_service.mjs` passed (5 checks). `git diff --check` passed.
- A disposable PostgreSQL test database seeded with account/passkey tables and Roarlay migration versions 1–8 accepted the service migrations under a restricted billing role. Stripe created two entries in `rawrarr_stripe_migrations`; the Roarlay ledger remained at eight. The restricted role had no SELECT privilege on `rawrarr_passkeys` and could not read `_sqlx_migrations`.
- Health returned 503 without a webhook-secret file and 200 with a populated file. Repeated startup kept both migration counts unchanged.
- `cargo fmt --check` was unavailable because the OrbStack test image does not include `cargo-fmt`.
- Published `main` commit `165b9b0b149f17d8adf8ddfc2c1e8398632050e3` was built on `rawrarr-roarlay` as `linux/amd64` image `sha256:7f6539604372189cdfa85e6b546a57db53fbf285c0ba11b285353607ece41c49`; the image revision label matches the published commit.
- Deployed `rawrarr-stripe-service:do3` as `rawrarr-roarlay-prod-stripe-service` on the existing control network with no published host port. Only the `stripe-service` Compose service was created; Roarlay, PostgreSQL, Headscale, Tailscale, and Caddy stayed running.
- Added only `/v1/webhooks/stripe` to the `rawrarr.willjasen.net` Caddy site and reloaded Caddy after validation. Caddy validation passed before and after reload. Public `POST /v1/webhooks/stripe` returned 503, which confirms the path reaches the new service while its test signing secret is intentionally empty. The public `/health` route continued to serve the Roarlay health page.
- The production database now has two rows in `rawrarr_stripe_migrations`; Roarlay's `_sqlx_migrations` remains at eight. `rawrarr_stripe` can select `rawrarr_users.id` and has no SELECT privilege on `rawrarr_passkeys`.
- The host-local files `/opt/rawrarr-roarlay-prod/secrets/stripe-db-password` and `/opt/rawrarr-roarlay-prod/secrets/stripe-webhook-secret` are owned by UID/GID 10001 and mode 0400. The webhook-secret file is empty. The service logs that its secret is unconfigured and returns 503 until Will creates the Stripe test-mode endpoint and installs its signing secret. Invalid-signature verification and accepted test events remain pending that secret; do not send live payments.
