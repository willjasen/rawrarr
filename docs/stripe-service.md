# Stripe billing service boundary

The Stripe webhook service runs only on the hosted service node. It connects to the hosted account PostgreSQL database containing `rawrarr_users` (the account table to which passkeys are linked). It never connects to subscriber-node databases and never reads or writes `rawrarr_passkeys`.

## Account linkage

Billing ownership is keyed by the existing `rawrarr_users.id`. `rawrarr_stripe_customers.account_id` is a one-to-one foreign key to that account row; `rawrarr_stripe_subscriptions.account_id` references the same owner. Thus billing information is associated with the passkey-owning account without copying passkey credentials into the billing service. The service stores Stripe customer and subscription identifiers, subscription status, period end, cancellation flag, and minimal webhook event metadata. Card details and payment methods remain with Stripe.

For first-time customer association, trusted hosted checkout must set Stripe object metadata `rawrarr_account_id` to the authenticated account UUID. The webhook handler rejects malformed UUIDs and refuses to reassign an already-mapped Stripe customer. Later subscription and invoice events can resolve ownership through the customer mapping. Billing events without a resolvable account are rejected and are not recorded, so they cannot become candidates for entitlement.

## Deployment and limits

Build with `docker build -f stripe-service/Dockerfile -t rawrarr-stripe-service .`. Configure `BILLING_DATABASE_URL`, `STRIPE_WEBHOOK_SECRET`, and optionally `BILLING_LISTEN_ADDR` (default `0.0.0.0:8090`). The target account database must already have the Rawrarr authentication schema (`rawrarr_users`). The service migrations add only billing tables. Never point it at a subscriber node's PostgreSQL database.

Keep webhook secrets in the service deployment's secret manager or mounted secret file and pass them at runtime. Do not put them in this repository, browser code, subscriber configuration, `/shared`, or the image.

The service verifies the raw Stripe payload signature with a five-minute timestamp tolerance, accepts selected checkout/subscription/invoice events, and commits event deduplication plus account/customer/subscription updates transactionally. Subscription rows are updated only by events at least as recent as the stored Stripe event timestamp. This is billing-state intake, not an entitlement: Checkout Session creation, customer portal, reconciliation, license issuance, and feature enforcement remain unimplemented. Do not activate paid access based only on these rows.
