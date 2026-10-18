# Rawrarr 24-hour trial direction

## Objective and initial state

Record the user's trial requirement: the person who sets up Rawrarr may use it for 24 hours, after which the instance stays locked until subscribed. Existing product notes say subscription terms are not defined; see [private self-hosted product direction](2026-10-17-rawrarr-private-self-hosted-direction.md), [shared instance identity](2026-10-17-rawrarr-shared-instance-id.md), and [product direction](../docs/product-direction.md).

## Proposed behavior

- Start the one-time trial when the owner completes first-run passkey and age-identity setup and first reaches the usable dashboard. This avoids consuming the trial before the app can be used.
- Store the trial start durably; restarts and clock changes in the app process must not reset it.
- Attach the trial/subscription entitlement to the shared Rawrarr `instance_id`, not an individual `device_id`, so all nodes sharing the instance see the same remaining trial and paid state.
- After 24 hours without an active subscription, gate product features but leave sign-in, subscription recovery, and data export available. Do not delete or encrypt additional user data as a lockout mechanism.
- Verify subscription through a signed entitlement from a licensing/billing control plane separate from the blind relay. Send only instance/license identifiers and entitlement state; never send library data, age identity, or Rawrarr config.
- A self-hosted owner controls the host and binary, so local trial enforcement cannot be tamper-proof. Treat it as a subscription gate for legitimate use, not a guarantee against binary/clock/database modification.

## Remaining decisions and verification

No trial or paywall behavior is implemented. Select the checkout/licensing provider, offline validation/grace behavior, and data-export path during lockout before implementation. The setup-completion point and per-instance scope above are proposed defaults. `git diff --check` and relative-link checks passed.
