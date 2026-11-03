# Start the isolated Stripe webhook intake service

## Objective

Start Stripe work under [billing issue #160](https://github.com/willjasen/rawrarr/issues/160) while keeping provider secrets and billing records out of subscriber coordinator/node code paths.

## Implemented

- Added a separate Rust `stripe-service` package with its own Dockerfile, PostgreSQL migration, and HTTP routes.
- Verify Stripe's HMAC-SHA256 `Stripe-Signature` header against the exact raw body, including the five-minute timestamp tolerance and multiple `v1` signatures used during signing-secret rotation.
- Accept a selected allowlist of subscription/checkout/invoice events, deduplicate by Stripe event ID, and store only event ID/type/provider timestamp/receipt timestamp.
- Link Stripe customers and subscriptions to the existing hosted `rawrarr_users.id` that owns passkeys. Use authenticated account metadata for first association and customer mapping for later events; reject malformed metadata and prevent reassignment. The service never reads the passkey table.
- Ignore unsupported signed event types and bound request bodies to 1 MiB. Keep the service source and database separate from subscriber deployments.
- Document remaining hosted checkout creation, account portal, instance registration, and signed-license work in [the Stripe service guide](../docs/stripe-service.md).

## Limits and follow-up

This slice does not create Checkout Sessions or Customer Portal sessions or issue entitlements. Do not use it to activate paid access. It has not been deployed or connected to a live/test Stripe endpoint. The service requires the hosted account schema (`rawrarr_users`) and must never target a subscriber database. Checkout should wait for a trusted hosted account portal and stable `instance_id` registration; do not expose service credentials in subscriber browser or node configuration.

## Verification

`cargo test --offline` passed in the isolated `rawrarr-testing-base:local` OrbStack container (7 Rust tests). Static regression checks and DB-backed Stripe test-mode webhook delivery remain to be exercised.
