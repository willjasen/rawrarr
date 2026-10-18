# Donation plan

## Goal

Let people support rawrarr with an optional donation in addition to subscribing. A donation is a voluntary contribution, not a purchase, subscription, license, or access grant. The app should remain usable without donating, and donating must not unlock subscriber features or change a subscriber instance's entitlement.

This is a product and implementation plan only. No donation provider, checkout, contribution tracking, or UI is implemented.

## Product behavior

- Offer **Donate** as a separate action alongside the existing subscription path. Explain briefly that a donation supports ongoing development and operations.
- Support a one-time contribution first. Let the payment provider handle payment details, confirmation, and receipts; rawrarr should not collect card or bank details.
- Prefer provider-hosted checkout with a small set of suggested amounts and an optional custom amount, subject to provider capabilities and local legal/tax review.
- Do not require a rawrarr account, passkey, age identity, coordinator URL, or subscriber-node enrollment to donate. The donation flow must work for someone who has not installed rawrarr.
- Make recurring support a later, explicitly opt-in product decision. Never convert a one-time donation into a recurring charge or imply that subscription renewal is a donation.
- Do not offer technical support, service levels, feature voting, storage, relay capacity, or entitlement extensions as donation rewards. Any future benefit needs its own explicit product and legal review.
- Provide a clear return path and thank-you state after successful checkout. Cancellations and failures should be neutral and easy to leave.

## Separation from subscriptions and licenses

Keep donation records and checkout separate from the subscription/license service described in [the subscription model](subscription-model.md):

- Donation payment success must not issue, extend, renew, or alter a signed license, trial, grace period, or account role.
- Subscription checkout must not silently request or create a donation.
- A donor may later subscribe through the normal flow; the donor identity and Rawrarr account should not be joined unless the person deliberately chooses that and a concrete feature requires it.
- Do not send donation status, amount, or donor contact details to a coordinator, metadata node, storage node, relay, SwarmSail, or subscriber database.
- Keep donation and subscription reporting distinct in operator accounting and user-facing language.

## Proposed flow and architecture

1. Publish a donation page on the public rawrarr site, reachable without signing in or running the app. The app may link to this page from About or Settings, but must not block setup or normal use.
2. The page explains the voluntary purpose, one-time nature, currency, amount, payment processor, receipt handling, and whether the contribution is refundable or tax-deductible. Avoid claims until verified for the operator's legal/tax status and jurisdiction.
3. The browser asks a small donation backend to create a provider-hosted checkout session using an allowlisted amount/currency and a non-sensitive random checkout reference. Never accept client-supplied entitlement fields or subscriber IDs.
4. The backend creates a one-time payment session. The provider handles payment data and returns the donor to a success or cancellation page.
5. A signed provider webhook may record payment state for reconciliation and aggregate reporting. Verify signatures, make event handling idempotent, minimize retained fields, and never trust the browser redirect as proof of payment.
6. Send receipts through the provider. Do not retain payment credentials. Retain only what is needed for reconciliation, accounting, fraud handling, and legal obligations, with documented access and retention limits.

The donation backend is an operator-side public service, separate from subscriber service nodes and the blind relay. It must not receive age identities, private keys, media data, app configuration, API keys, or subscriber-node credentials. Prefer aggregate totals in public reporting; do not publish donor names or amounts without separate explicit consent.

## Privacy and security requirements

- Use the payment provider's hosted checkout; keep card data outside rawrarr infrastructure.
- Collect no rawrarr account or node identifiers by default. Do not add advertising pixels or cross-site tracking to checkout.
- Keep webhook secrets and provider credentials in the operator's secret store, never in the repository, browser bundle, Compose file, or logs.
- Validate amount, currency, checkout reference, redirect URLs, and webhook event type on the server. Use CSRF protections where applicable, rate limits, and abuse monitoring that avoids logging personal payment data.
- Do not place donor email, name, provider payloads, or payment references in application logs or public analytics. Restrict accounting records to authorized operators.
- Define deletion and retention rules before launch, including provider dashboard access and backups. Accounting obligations may limit deletion; disclose the actual policy.

## Decisions before implementation

- Select the operating legal entity and confirm where donations can be accepted.
- Choose a payment provider after checking supported countries, currencies, fees, hosted checkout, webhook signing, receipts, refunds, dispute handling, and data-processing terms.
- Decide whether contributions are described as donations, tips, or contributions based on legal/tax review; do not imply tax deductibility unless confirmed.
- Set suggested amounts, custom-amount floor/ceiling, supported currencies, refund policy, and accounting treatment.
- Choose whether donor contact information is optional for a receipt and whether the provider can issue receipts without the operator retaining the address.
- Define the minimal reconciliation record, access controls, retention period, webhook recovery procedure, and operational owner.
- Decide where the public page lives and whether the authenticated app should link to it.

## Delivery sequence

1. Confirm legal entity, donation terminology, jurisdictions, accounting and refund rules.
2. Compare providers against the requirements above and select one; keep the choice out of subscriber-node configuration.
3. Specify the public donation page copy, accessible amount selection, hosted checkout, success/cancel states, and privacy notice.
4. Implement the isolated donation-session endpoint and signed, idempotent webhook recording with secret-safe logs.
5. Add reconciliation, refund/dispute handling, retention, backup, and operator access procedures.
6. Add a non-blocking link from About or Settings only after the public flow is ready.
7. Verify one successful payment, cancellation, declined payment, duplicate/out-of-order webhook, refund, receipt delivery, and that no license or subscriber data changes.

## Acceptance criteria

- A person can make a one-time contribution without a Rawrarr account or installation.
- Checkout clearly identifies the amount, currency, recipient/operator, payment processor, and contribution terms before payment.
- No payment instrument data passes through rawrarr services.
- Verified payment events are recorded once for reconciliation; duplicate webhook delivery does not duplicate a contribution.
- Donation success, failure, refund, or dispute never changes trial or license state.
- Subscriber nodes, coordinator databases, relay paths, shared files, and age-encrypted data contain no donation data.
- Donor details are not made public and are retained only under the disclosed, approved policy.
- The donation option is clearly optional and does not interrupt or restrict product use.

## Related plans

- [Subscription model](subscription-model.md) — trial, hosted subscription checkout, and signed entitlements.
- [Product direction](product-direction.md) — Rawrarr scope and privacy boundaries.
- [Security](SECURITY.md) — service and subscriber trust boundaries.
