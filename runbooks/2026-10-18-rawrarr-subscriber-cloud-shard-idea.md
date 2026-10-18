# Consider a private cloud shard for subscribers

## Objective

Preserve a future product idea: when someone subscribes, Rawrarr could provision a private cloud node or shard for that subscriber, with a defined disk-space quota. Vult is a possible hosting provider to evaluate.

## Direction recorded

- The cloud resource would be private to the subscriber and provisioned as part of subscription onboarding or activation.
- Storage would have an explicit quota; pricing, quota sizes, and behavior when the quota is reached are undecided.
- Vult is a candidate provider, not a selected or verified hosting choice.
- This is an idea for later exploration. It does not change the current self-hosted subscription direction or authorize provisioning cloud infrastructure.

Related planning: [subscription model](2026-10-18-rawrarr-subscription-model.md) and [cloud access architecture](2026-10-18-rawrarr-cloud-access-architecture.md).

## Open questions

- Would the shard store app data, encrypted backups, cached metadata, or another explicitly scoped data set?
- How would subscriber ownership, encryption keys, backup, deletion, and migration work?
- What quota tiers and over-quota behavior make sense, and how would storage costs affect subscription pricing?
- Does Vult provide the isolation, regions, operational controls, and pricing this design needs?

## Final state

The idea is recorded for future product and architecture review. No code, subscription behavior, hosting account, or cloud resource was changed or created.
