# Rawrarr subscription model

## Purpose and current decisions

Rawrarr is intended to be self-hosted from a packaged Docker image. A new account on a public deployment receives a three-day trial; internal deployments do not receive a billing trial. The public coordinator enforces a full-access three-day trial, then a permanent free tier with at most one fossil (metadata) node, and a paid tier that lifts the fossil cap while active. After trial expiry, the effective tier falls back to free. Sign-in, recovery, export, and deletion remain available, and tier changes never delete subscriber data. The current implementation gates fossil enrollment and approval at the coordinator; it does not yet issue signed offline entitlements to subscriber nodes.

Voluntary ETH donations do not require an account and never change the account tier. The first ETH subscription flow is a manual Ethereum-mainnet renewal; automatic recurring payments require a later, separately audited contract. Treasury address and exact plan amounts must be deliberately configured before their corresponding payment controls are enabled. ETH payment verification and coordinator entitlement are implemented; CCIP Read and smart-account signatures are not yet supported. Stripe remains a separate integration workstream and does not change ETH donation or subscription verification. See [ENS identity and ETH payments](crypto-identity-and-payments.md) and the [product/design decisions](DESIGN.md#account-tiers-ens-sign-in-and-voluntary-eth).

## Proposed commercial shape

### Bill the shared instance

Attach one subscription to the Rawrarr `instance_id`, which represents the shared app-data folder. Do not bill by container or device: a coordinator, optional shards, and several storage nodes are all part of one installation, and node-count billing would make the distributed design harder to use. A person who creates the instance is its billing owner. Invited people and devices use that instance's entitlement; they do not create extra subscriptions just by pairing.

This leaves a future product decision: whether collaborator count or optional hosted services belong in higher tiers. Keep those limits out of the first license schema until the product has a clear reason to meter them.

### Keep the first catalog small

Start with one paid self-hosted plan, offered monthly and annually once pricing is decided. The plan unlocks the full self-hosted product for one instance and its participating nodes. Consider an optional relay service or collaboration tier later, after their operating costs and access rules are understood. Avoid a multi-tier matrix before Rawrarr has stable feature boundaries.

A new account on a public deployment receives one three-day evaluation, beginning when passkey registration completes. PostgreSQL records its start timestamp on the new account row; restarting the coordinator does not restart it. Internal deployments set `RAWARR_BILLING_AUDIENCE=internal` and do not record or expose trial timing. Existing accounts are left unset by the migration. The current implementation reports the start and end in the authenticated profile but does not yet gate features after expiry. First-use enforcement can be bypassed by a host owner who controls the container and its data, so it is a product gate rather than tamper-proof licensing.

## Proposed purchase and license flow

1. Rawrarr creates a random instance identity and starts its trial after setup is complete.
2. The owner selects **Subscribe** in the app and goes to a hosted checkout page. Payment details stay with the billing provider.
3. The hosted account portal creates a Stripe customer linked to the authenticated `rawrarr_users.id`. The separate Stripe service verifies selected provider events, records an idempotent event inbox, and maintains customer/subscription references against that same account row. A later licensing worker must associate the active plan with the opaque instance ID.
4. The licensing service issues a signed entitlement. The app stores the signed payload in shared Rawrarr app data so connected nodes can verify the same entitlement without sharing a database.
5. Every node verifies the signature locally against a public verification key packaged with Rawrarr. The app does not need the license-signing private key or billing credentials.
6. Nodes periodically refresh entitlement state. When the licensing service is unavailable, a previously valid signed entitlement continues to work through a proposed 14-day renewal grace window.

Keep the entitlement payload small and free of personal billing details. Candidate fields are schema version, opaque instance ID, plan identifier, feature identifiers, issue time, paid-through time, renewal/grace deadline, and signing-key identifier. Do not include payment method data, email address, media inventory, age identity, or Rawrarr configuration. The license file is signed, not a secret; avoid implying that a signature encrypts it. Select a post-quantum signature scheme and a key-rotation procedure before implementation so the license trust model matches Rawrarr's post-quantum goals.

## Trial expiry, renewal, and offline behavior

- During the three-day trial, the instance has the intended paid feature set except that each signed-in account may submit at most five AI questions per UTC calendar day across text and voice. Roarlay enforces this using its local PostgreSQL request records. Show the remaining allowance and local-time reset; do not count failed requests without a usable result, internal model/tool turns, or owner review-agent processing. The initial AI implementation applies this limit to all accounts; connecting this allowance to the new trial record and enforcing paid entitlements remain planned.
- At trial expiry with no valid subscription, gate product workflows. Always retain passkey sign-in, subscription recovery, data export, backup recovery, and account/data deletion paths. Preserve the user's data; never delete or re-encrypt it as a payment enforcement action.
- On cancellation, retain access through the provider-confirmed paid-through date. On a failed renewal, allow the proposed 14-day grace period before gating features.
- An offline node can use the most recent signed entitlement until its grace deadline. Do not treat a temporary billing-service outage as cancellation.
- If an entitlement is expired, malformed, for a different instance, or signed by an unknown key, do not unlock paid workflows. Keep recovery/export paths available and explain the state in the UI.
- Do not let a stale entitlement from Syncthing extend access beyond its signed deadline. Clock rollback is a known limitation on self-hosted systems; use persisted highest-seen time as a basic safeguard, not as a tamper-proof clock.

The proposed 14-day window is a starting point for discussion, not a committed policy. Decide whether read-only access to the library and metadata remains available after expiry; the existing direction says the product is locked but requires recovery and data preservation.

## Privacy and service boundaries

| Component | Needs to know | Must not receive |
| --- | --- | --- |
| Hosted checkout / payment provider | Billing identity and payment details required for checkout | Media inventory, age identity, Rawrarr config, node contents |
| Licensing service | Provider subscription reference, opaque instance ID, plan and entitlement status | Media files, metadata catalog, age identity, config plaintext, passkey records |
| Relay/subscription account service | Passkey account records, public age recipient(s) linked to the account, billing reference, opaque instance/device IDs, subscription status | Private age identity, subscriber app-data plaintext, media files, config plaintext, node database |
| Self-hosted Rawrarr node | Signed entitlement and public verification key; its own local app data | License-signing private key or payment credentials |
| Blind relay transport | Device/instance routing identifiers and transient connection state; opaque encrypted stream bytes in flight | Private age identity, billing records, subscriber app-data plaintext, media or config payloads |

The hosted service may associate public age recipients with the account's passkeys, but a public recipient is not an authentication factor. Never send the corresponding private identity to the relay or subscription service. The relay does not encrypt, decrypt, or store subscriber app-data payloads; it only forwards opaque end-to-end encrypted bytes. Keep billing identifiers out of the Syncthing shared folder. The shared signed entitlement can contain the opaque instance ID and plan status only; no contact or payment information.

## Decisions before implementation

1. Confirm the instance as the billing unit and decide whether invites count as seats.
2. Choose price and monthly/annual terms after estimating support, relay, and payment costs.
3. Confirm whether the post-expiry experience is hard lock, read-only library, or a narrower feature gate; preserve sign-in, export, and recovery in every case.
4. Choose the checkout/payment provider and define the licensing service's responsibility and availability target.
5. Confirm the offline grace duration and behavior for cancellation, failed payment, chargeback, refund, and provider outage.
6. Select a post-quantum signature scheme, license format, signing-key custody, rotation/revocation, and app upgrade path.
7. Decide whether licenses are refreshed online, distributed only through shared app data, or use both paths.
8. Define account recovery and license transfer when an owner loses access to the instance or intentionally forks it.
9. Threat-model local clock rollback, copied instance folders, cloned containers, and deliberate binary modification; accept that self-hosting prevents perfect enforcement.

## Implementation sequence

1. Finalize product rules and write a feature/lock-state matrix.
2. Define versioned trial and entitlement records keyed by `instance_id`, with safe conflict handling under eventual synchronization.
3. Build on the separate Stripe webhook inbox with event processing and signed license issuance; keep billing and licensing separate from the blind relay.
4. Add hosted checkout and customer-portal links without collecting card data in Rawrarr.
5. Add local signature verification, shared-folder license storage, refresh/grace handling, and key rotation.
6. Gate server-side API mutations and workflows, not only UI buttons; keep recovery/export routes available.
7. Test trial expiry, valid/invalid signatures, wrong instance IDs, renewal, cancellation, grace, outage, stale Syncthing copies, clock rollback, and data preservation.
8. Document self-hosted installation, billing recovery, license transfer, privacy disclosures, and support procedures.

## Related decisions

- [Product direction](product-direction.md) — self-hosted distribution and overall trust boundaries.
- [Device architecture](device-architecture.md) — shared instance and device identity.
- [Data domains](data-domains.md) — shared app-data and local storage boundaries.
