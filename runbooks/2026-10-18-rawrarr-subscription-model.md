# Plan a Rawrarr subscription model

## Objective and initial state

Turn the existing direction—24-hour trial, self-hosted deployment, hosted checkout, and a signed license—into a reviewable product and technical proposal. No checkout, payment provider, pricing, license verification, or paywall behavior is implemented.

## Actions

- Reviewed [product direction](../docs/product-direction.md), [24-hour trial direction](2026-10-17-rawrarr-24-hour-trial.md), [private self-hosted direction](2026-10-17-rawrarr-private-self-hosted-direction.md), and [shared instance identity](2026-10-17-rawrarr-shared-instance-id.md).
- Proposed billing one shared `instance_id`, with participating Rawrarr nodes included and no initial per-device charge.
- Documented a hosted-checkout to separate licensing-service flow, signed entitlement distribution, post-quantum signature selection, and an initial 14-day offline renewal grace proposal.
- Recorded privacy boundaries, trial/expiry/renewal behavior, open product decisions, and an implementation sequence in [the subscription model](../docs/subscription-model.md).

## Verification and final state

Documentation-only planning. No payment provider, app behavior, containers, or billing service were changed. `git diff --check` passed, and relative links in the four changed Markdown files resolve. Pricing, tier limits, the post-expiry read-only policy, grace duration, signature algorithm, and provider remain undecided.
