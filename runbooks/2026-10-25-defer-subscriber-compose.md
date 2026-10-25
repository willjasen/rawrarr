# Defer subscriber Compose deployment work

## Objective and initial state

Record the remaining optional subscriber Compose installer work for a later task, and clean up the completed first-account onboarding branch. GitHub `main` already contains [PR #60](https://github.com/willjasen/rawrarr/pull/60), which lets an owner finish passkey, age ID, and signing-key setup on Devices without deploying a node. [Issue #34](https://github.com/willjasen/rawrarr/issues/34) tracks the Compose install gaps. The primary `main` checkout had extensive unrelated modifications and a local commit, so this documentation change used a fresh worktree from `origin/main`.

Relevant earlier work: [subscriber node onboarding](2026-10-18-subscriber-node-onboarding.md) and [simplify first-account onboarding](2026-10-22-simplify-first-account-onboarding.md).

## Actions

- Added a deferred installer item to `ROADMAP.md`, covering the unnecessary subscriber GitHub token, unavailable documented localhost sign-in address, unpublished application image, supported-host validation, and subscriber-only role boundary.
- Preserved the merged first-account flow. No runtime or deployment changes were made.
- Closed completed [issue #57](https://github.com/willjasen/rawrarr/issues/57) with a link to PR #60 and the deferred installer issue.
- Deleted the merged `codex/onboarding-compose-devices` branch locally and from `origin`. No worktree was attached to that branch.

## Verification and final state

Verified PR #60 is merged in GitHub `main`, issue #57 is closed, and issue #34 remains open for the deferred installer. Reviewed the three-file documentation diff and `git diff --check` passed. No runtime checks were run for this roadmap-only change.

## Recovery and remaining work

Remove only this task's roadmap entry and runbook if the planning decision changes. Implement [issue #34](https://github.com/willjasen/rawrarr/issues/34) in a later issue-scoped branch; do not treat the current Compose preview as a complete installer.
