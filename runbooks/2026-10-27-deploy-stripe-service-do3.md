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
- The production Caddy route and do3 container are not deployed yet. After merge and deployment, verify Caddy routes `/v1/webhooks/stripe`, invalid signatures are rejected, and the service becomes healthy only after the Stripe test endpoint's signing secret is configured. Do not send live payments during this test deployment.
