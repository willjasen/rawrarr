# Rawrarr Stripe billing service

This is an isolated hosted service-node component. It accepts signed Stripe webhooks, verifies the `Stripe-Signature` header, and associates Stripe customer/subscription references with the account row that owns linked passkeys. It does not read or write passkey records, store webhook bodies, customer contact details, payment methods, age identities, media, or Rawrarr app data.

## Run locally

Build with `docker build -f stripe-service/Dockerfile -t rawrarr-stripe-service .` from the repository root. Configure `BILLING_DATABASE_URL`, `STRIPE_WEBHOOK_SECRET`, and optional `BILLING_LISTEN_ADDR` (defaults to `0.0.0.0:8090`). The connection targets the hosted account database containing `rawrarr_users` and its linked passkeys. The service migrations add billing tables there; never point it at a subscriber node's PostgreSQL instance. `/health` checks the database; configure Stripe to send the selected subscription and invoice event types to `/v1/webhooks/stripe` through a TLS-terminating service-node ingress.

Keep webhook secrets in the service deployment's secret manager or mounted secret file and pass them to the process at runtime. Do not put them in this repository, browser code, subscriber configuration, `/shared`, or the service image.

## Current limits

It does not create Checkout Sessions, customer portal links, or signed entitlements. Do not use it to activate paid access. Checkout needs the hosted account portal to set `rawrarr_account_id` metadata from the authenticated `rawrarr_users.id`; the webhook handler refuses to reassign an existing Stripe customer to another account. Signed licenses still require stable shared `instance_id` registration, product/pricing configuration, and license-signing design.
