# Stale GitHub branch cleanup

Issue: #385

## Scope and safeguards

Remove remote branches only when their current tip exactly matches a merged
pull request. Preserve the default branch, protected branches, branches with
open pull requests, and branches checked out in local worktrees. Leave
ambiguous or unmerged branches untouched.

The initial inventory identified 113 eligible branches and 11 excluded branches.
Recheck remote tips, open pull requests, and local worktrees before deletion.
Use ordinary Git remote-ref deletion without force-pushing or modifying local
branches and worktrees.

GitHub's ruleset endpoint returned HTTP 403 because rulesets require a different
repository plan. The branch inventory still exposes each branch's protected
flag; protected branches are excluded.

## Adoption and recovery

This is repository maintenance only. Existing deployments, persistent data,
schemas, and protocols are unchanged; no migration is needed.
Merged pull requests retain the deleted branches' commit references. If a
deleted branch is needed again, inspect its merged pull request and recreate
only that branch from its recorded head commit.

## Result

Deleted 113 eligible remote branches after a successful Git push deletion dry
run and a fresh check of remote tips, open pull requests, and local worktrees.
A fresh GitHub branch inventory confirmed all 113 were absent, all 11 excluded
branches remained, and the local worktree inventory was unchanged.

Retained branches:

- `main`: default branch.
- `codex/internal-coordinator-backup`: open PR #300.
- `willjasen-fix-fossils-loading`: local worktree; merged PR #368.
- `willjasen-fix-movies-initial-load`: local worktree; merged PR #375.
- `codex/analytics-dashboard-basefix`: tip does not match a merged PR head.
- `codex/analytics-internal-dashboard`: tip does not match a merged PR head.
- `codex/fix-group-age-recipient-mismatch`: closed, unmerged PR #170.
- `codex/internal-arr-path`: no matching PR in the inventory.
- `codex/netlify-ignore-landing-changes`: closed, unmerged PR #337.
- `codex/passkey-rawrarr-rp-id`: tip does not match a merged PR head.
- `codex/stripe-migration-isolation`: tip does not match a merged PR head.

No runtime suites were run; this task changes only remote Git refs and this
maintenance record.
