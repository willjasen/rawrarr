# Account-bound subscriber tailnet provisioning plan

Status: planned. The coordinator-side signup and subscriber provisioning implementation has not been integrated into `main`.

## Goal

Give each eligible Rawrarr account an isolated private Headscale-managed tailnet and issue subscriber client configurations only for that account. Keep the operator service outside subscriber app data and preserve Rawrarr's own node authentication and sharing rules; tailnet membership is transport, not authorization to decrypt shared data.

## Proposed flow

1. A passkey-authenticated account becomes eligible under a separately defined account policy. Signup records a durable provisioning request; subscriber nodes cannot create or select accounts.
2. A private coordinator-to-SwarmSail service call requests provisioning using an opaque account ID and a preferred stable name. Server-side account lookup verifies eligibility; callers cannot assert it themselves.
3. Provisioning is asynchronous and idempotent. Distinguish queued, provisioning, ready, and failed states. Network timeouts are unknown outcomes, so retries reuse the same account identity and do not create duplicate tailnets.
4. Publish subscriber client configuration only after the service confirms the same account is ready. Bind client keys to that account and deliver them through an authenticated owner flow; never place reusable credentials in static Compose, logs, or synchronized app data.
5. On account or device revocation, revoke affected clients and preserve subscriber data. Entitlement expiry must not silently delete user data.

## Boundaries

- The operator service may know account IDs, eligibility, tailnet allocation, device IDs, and provisioning status. It must not receive media, plaintext shared configuration, private age identities, node-local databases, or subscriber credentials unrelated to tailnet enrollment.
- Use a private authenticated service API with separate credentials for provisioning and account-eligibility callbacks. Keep admin APIs off public routes.
- Apply a restrictive per-tailnet ACL and verify it before reporting provisioning complete. The existing roadmap's minimum app connectivity allowance is TCP 443 and Syncthing TCP 22000; media paths and broad host/LAN access remain out of scope.
- Keep node authentication, account ownership, age recipients, and content-sharing grants enforced by Rawrarr independently of network membership.

## Decisions before implementation

- Whether isolation means one Headscale server per account or separate organizations/tailnets on shared Headscale, including capacity and failure-domain tradeoffs.
- What makes an account eligible, how trial/subscription state is authoritative, and what happens after lapse or cancellation.
- Stable public naming, collision handling, multiple instances per account, and collaborator access across different account tailnets.
- Client key rotation, revocation, recovery, and secure owner delivery without leaking credentials into process arguments or Compose files.
- Operational behavior for partial provisioning, retries, service outages, and account deletion.

## Acceptance checks

- Concurrent signup/retry requests create at most one account-owned tailnet allocation.
- The provisioning service verifies eligibility from the coordinator and rejects mismatched account IDs or allocations.
- No subscriber client configuration is available before provisioning is ready; issued clients belong only to the intended account.
- ACL tests deny unapproved ports and host/LAN access while allowing only the documented app paths.
- Revocation and entitlement-lapse tests remove network access without deleting subscriber data or granting access to shared age-encrypted content.

See [cloud access architecture](cloud-access-architecture.md), [Tailscale integration](tailscale-integration-plan.md), and the [subscription model](subscription-model.md) for related boundaries and open product decisions.
