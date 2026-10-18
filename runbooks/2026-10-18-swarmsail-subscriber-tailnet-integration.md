# Integrate subscriber tailnets with Swarmsail

## Objective and initial state

Connect Rawrarr account eligibility to dedicated Swarmsail tailnets, with a one-way private-to-shared transition. Reviewed [tailnet account boundary](2026-10-18-rawrarr-swarmsail-tailnet-account-boundary.md), [node types](../docs/node-types.md), [subscription model](../docs/subscription-model.md), and [isolated/shared cluster design](2026-10-18-isolated-to-shared-cluster-design.md). The local Rawrarr coordinator has node-local passkey accounts but no hosted account, verified subscription event, signed entitlement, or Swarmsail API client. Its current Tailscale overlay uses a separate live tailnet.

## Decisions and work

- Provision one isolated Headscale instance per person only after a verified active trial or subscription. Sign-in alone does not create one.
- The person's tailnet may move to shared, never back to private. A new private tailnet requires a new network identity and device reenrollment.
- Tailnet mode is separate from Rawrarr's encrypted-data cluster mode. Network reachability does not grant Rawrarr access or age decryption.
- Swarmsail now has a local one-way organization transition and confirmation UI. It is not deployed or connected to Rawrarr sign-in.

## Verification and remaining work

The Swarmsail SQLite transition passed focused local tests. No hosted sign-in, entitlement verification, tailnet provisioning, Headscale policy, cross-tailnet route, or live deployment was verified. Implement the hosted identity and eligibility service, account-to-tailnet provisioning, owner authorization, enrollment, and cross-instance access policy before claiming the end-to-end flow works. Preserve the existing live deployment and subscriber data while doing so.
