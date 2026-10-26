# Show identity domains in the coordinator About page

Issue [#95](https://github.com/willjasen/rawrarr/issues/95).

## Objective
Make the coordinator app explain the agreed user, device, group, and app identity domains and distinguish available behavior from planned behavior. This follows the documentation-only completion of #95 in [PR #128](https://github.com/willjasen/rawrarr/pull/128); the issue was reopened for the missing in-app explanation.

## Initial state
Started from fetched `origin/main` at `8b4776a` in `/Users/willjasen/.codex/worktrees/issue-95-identity-ui/rawrarr` on `codex/issue-95-identity-ui`. The About page described sign-in, node selection, Arr configuration, and library browsing, but omitted identity domains. The identity guide incorrectly left per-app key access open to per-node authorization even though the agreed initial model gives every approved group node every app identity.

## Actions
- Added a responsive four-card identity overview to the coordinator About page, labeling user/device behavior current and group/app behavior planned.
- Explained that group config will use the group age identity, per-app age IDs are created by Rawrarr for that app's metadata only, all approved group nodes receive them for now, and migrations preserve app identity.
- Clarified age encryption versus ML-DSA signatures and that planned user/node messaging remains age-encrypted while Tailscale Serve carries ciphertext only.
- Updated the identity guide to record the agreed all-approved-nodes app-key access model.
- Reopened #95 and added the in-app acceptance criteria.
- Added focused static regression coverage to the existing coordinator workspace suite and updated its test index.

## Verification
- `node --test scripts/tests/test_rawrarr_coordinator_workspace.mjs` passed: 28 tests.
- `git diff --check` passed.
- No live coordinator deployment or browser screenshot verification was performed.

## Final state and remaining work
The source now presents all four identity domains and current/planned boundaries in About. The group/app lifecycle and encrypted messaging remain planned implementation work in #109–#112, with signed trust and metadata verification in #40. No deployment was performed.
- Committed source and runbook as `1bd7090` on `codex/issue-95-identity-ui`, pushed to `origin`, and opened [PR #131](https://github.com/willjasen/rawrarr/pull/131). The PR is open and mergeable; GitHub Actions Basic script tests passed. No merge or deployment was performed.
