# Stripe billing service boundary

The Stripe webhook service runs only on the hosted service node. It connects to the hosted account PostgreSQL database containing `rawrarr_users` (the account table to which passkeys are linked). It never connects to subscriber-node databases and never reads or writes `rawrarr_passkeys`.

## Account linkage

Billing ownership is keyed by the existing `rawrarr_users.id`. `rawrarr_stripe_customers.account_id` is a one-to-one foreign key to that account row; `rawrarr_stripe_subscriptions.account_id` references the same owner. Thus billing information is associated with the passkey-owning account without copying passkey credentials into the billing service. The service stores Stripe customer and subscription identifiers, subscription status, period end, cancellation flag, and minimal webhook event metadata. Card details and payment methods remain with Stripe.

For first-time customer association, trusted hosted checkout must set Stripe object metadata `rawrarr_account_id` to the authenticated account UUID. The webhook handler rejects malformed UUIDs and refuses to reassign an already-mapped Stripe customer. Later subscription and invoice events can resolve ownership through the customer mapping. Billing events without a resolvable account are rejected and are not recorded, so they cannot become candidates for entitlement.

## Deployment and limits

The do3 Compose overlay is [`deploy/do3/docker-compose.stripe-service.yml`](../deploy/do3/docker-compose.stripe-service.yml); its Caddy route fragment is [`deploy/do3/stripe-service.Caddyfile`](../deploy/do3/stripe-service.Caddyfile). Build with `docker build -f stripe-service/Dockerfile -t rawrarr-stripe-service:do3 .` from the published source snapshot. The service connects only to the hosted Rawrarr account database (`postgres://rawrarr_stripe@postgres:5432/rawrarr`) and reads its DB password and webhook secret from mounted files. The target database must already have `rawrarr_users`; the Stripe database role should have only CONNECT, schema USAGE/CREATE, and SELECT/REFERENCES on `rawrarr_users.id`. It must not have access to `rawrarr_passkeys` or subscriber databases.

Keep webhook secrets in the service deployment's secret manager or mounted secret file and pass them at runtime. `STRIPE_WEBHOOK_SECRET_FILE` and `BILLING_DATABASE_PASSWORD_FILE` are supported. If the webhook secret is empty or missing, the service remains unready and returns 503 for webhook requests until configured. Do not put secrets in this repository, browser code, subscriber configuration, `/shared`, or the image.

The service verifies the raw Stripe payload signature with a five-minute timestamp tolerance, accepts selected checkout/subscription/invoice events, and commits event deduplication plus account/customer/subscription updates transactionally. It keeps its migration ledger in `rawrarr_stripe_migrations`, separate from Roarlay's `_sqlx_migrations` table even though both services use the same account database. Subscription rows are updated only by events at least as recent as the stored Stripe event timestamp. This is billing-state intake, not an entitlement: Checkout Session creation, customer portal, reconciliation, license issuance, and feature enforcement remain unimplemented. Do not activate paid access based only on these rows.
