# Plan optional donations

## Objective and initial state

Plan an optional way for people to donate in addition to subscribing. The repository already has a proposed shared-instance subscription and license model, but no donation flow or provider is implemented. The worktree contains extensive unrelated edits; this task adds only a new standalone plan and links it from documentation indexes.

## Actions

- Reviewed the existing subscription model, product direction, product design, roadmap, documentation index, and runbook index.
- Defined donations as one-time and voluntary initially, independent of subscription entitlements and requiring no Rawrarr account or node identity.
- Added the provider-hosted checkout, isolated donation backend, webhook/reconciliation, privacy/security requirements, undecided launch decisions, sequence, and acceptance criteria to [the donation plan](../docs/donation-plan.md).
- Added the plan to the docs index and this task to the runbook index.

## Verification and final state

Documentation-only planning. No provider, checkout, payment service, product UI, license behavior, or deployment was changed. Verify Markdown links and whitespace before completion. Existing user changes remain untouched.

## Remaining decisions

Legal entity and jurisdiction, donation terminology, payment provider, amount/currency options, refund/accounting policy, retention limits, and public page location must be decided before implementation.
