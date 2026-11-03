# Stripe billing service

The Stripe service is separate from subscriber coordinator, storage, and metadata nodes. Its first implementation is a minimal webhook inbox in [`stripe-service/`](../stripe-service/README.md). It validates Stripe's timestamped HMAC signature against the exact request body, rejects signatures outside a five-minute tolerance, accepts only selected checkout/subscription/invoice events, and inserts event identifiers into a dedicated PostgreSQL table with a unique key. Stripe retries of the same event are acknowledged without creating another row. Unsupported signed event types are acknowledged and ignored.

The inbox stores only Stripe event ID, event type, Stripe creation time, and local receipt time. It does not persist webhook payloads, which can contain billing contact details. It does not yet process events, create checkout sessions, update subscription state, or issue signed entitlements. The service must not connect to subscriber databases or receive private age identities, passkeys, media, node contents, or synchronized app-data.

The service binds to port 8090 by default. The deployment must provide a dedicated billing database and `STRIPE_WEBHOOK_SECRET` as runtime configuration, then expose the webhook through TLS on the service node. No deployment or production webhook endpoint is configured by this change. `STRIPE_SECRET_KEY`, product price IDs, account portal authentication, billing ownership mapping, and the signed-license trust model are not implemented here.

Before adding checkout, connect the billing API to the authenticated hosted account portal rather than exposing a service token to subscriber browsers or self-hosted nodes. Use the shared `instance_id` as the billing key after its stable manifest/registration flow exists. Align any Stripe subscription trial end to the authoritative account/instance trial so users do not receive overlapping trial periods. Do not infer a trusted trial date from a request supplied by an unauthenticated browser.

## References

- [Subscription model](subscription-model.md) — trial, checkout, and entitlement direction.
- [Cloud access architecture](cloud-access-architecture.md) — service-node and subscriber boundaries.
