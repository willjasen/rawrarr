# Local work cleanup and integration plan

This plan records the remaining Rawrarr work found in local branches and worktrees on 2026-10-22. GitHub `main` is the integration baseline. Branch counts below are relative to the refreshed `origin/main` at `b023347`; they are snapshots, not proof that work is still current. Recheck branch contents, PR state, and working-tree changes before acting.

## Current local state

- `main` is checked out in the primary repository with 43 changed paths, including staged, unstaged, deleted, and untracked work. Treat all of it as preserved user work; do not clean or stage it wholesale.
- Two stashes remain: `stash@{0}` (`codex preserve pre-existing work before syncing origin/main`) and `stash@{1}` (`codex-preserve-before-pull-2026-10-22`). Their contents have not been reviewed in this task.
- Five feature branches remain from the earlier local inventory. Four are ahead of `origin/main`; one has an independent three-commit prototype. Two feature worktrees contain uncommitted changes.
- Previously discussed runbook-only cleanup candidates and the small profile-name UI branch are absent from the current local branch/worktree inventory. Do not recreate or remove anything based only on the older inventory.

## Deferred feature work

| Branch | Ahead / behind `origin/main` | Current worktree | Work to review and implement/integrate |
| --- | ---: | --- | --- |
| `willjasen-browser-only-private-access` | 46 / 167 | Clean | Large browser-only private access stack: independent browser client, encrypted peer access and supporting deployment/design changes. Compare with the overlapping browser-access implementation branch and choose one coherent integration path. |
| `willjasen-browser-access-implementation` | 52 / 167 | 9 modified paths | Browser identity handoff, root pinning, authenticated peer rendezvous, signed peer negotiation, read-only metadata RPC, ICE configuration, plus deployment and account-tailnet changes. Preserve and review the nine local edits before selecting commits. |
| `willjasen-do2-coordinator-migration` | 24 / 167 | 8 changed paths, including staged and untracked | Coordinator migration to the dedicated droplet, production Compose and Caddy setup, account-tailnet and enrollment changes, migration/recovery documentation. Review uncommitted migration-page, Compose, runbook, and test edits separately. |
| `willjasen-headscale-signup-integration` | 16 / 167 | Clean | Account-bound Headscale signup and subscriber tailnet client-key provisioning across Rust/API, migrations, Compose, tests, docs, and runbooks. Rebase or port only after resolving overlap with the coordinator migration stack. |
| `willjasen-rawrarr-node-rpc-prototype` | 3 / 167 | Clean | Node RPC and metadata replication prototype, persistent node state, and quorum status indicator. Decide whether this architecture remains desired; compare its node catalog assumptions with current main before integration. |

The browser-only access, browser-access implementation, Headscale signup, and coordinator migration branches overlap in deployment, account provisioning, and browser trust code. Review them together as a dependency group, but integrate in small issue-scoped PRs. Do not merge a whole branch solely because its worktree is clean.

## Safe cleanup sequence

1. Inventory every current worktree and stash again immediately before cleanup. For each branch, compare commit ancestry and file-level changes with then-current `origin/main` and check open PRs.
2. Review and preserve the uncommitted changes in `willjasen-browser-access-implementation`, `willjasen-do2-coordinator-migration`, and the shared `main` worktree. Record ownership and destination before moving any hunk.
3. Resolve the overlapping access/migration/tailnet work into focused GitHub issues and PRs. Verify the accepted changes have reached GitHub `main` before removing their local branches or worktrees.
4. Review `willjasen-rawrarr-node-rpc-prototype` independently against the current node architecture and decide keep, integrate, or abandon.
5. Inspect both stashes file by file. Preserve wanted work in a named branch or PR before dropping a stash.
6. Only after all unique commits and uncommitted edits have a verified disposition, remove stale worktrees and branches. Never use cleanup to discard unknown content.

## Acceptance criteria

- Every local branch, dirty worktree, and stash has a recorded disposition: integrate, preserve for later, or abandon with explicit confirmation.
- All desired changes have focused PRs and are verified on GitHub `main` before corresponding local copies are removed.
- No uncommitted work or stash content is lost during cleanup.
- The primary checkout's pre-existing changes remain intact unless separately selected for an authorized task.

## Evidence and limitations

- Refreshed `origin/main` and branch ancestry were inspected on 2026-10-22. The primary checkout was already eight commits behind its then-current origin ref and had extensive local changes.
- The previously linked voice task reported an earlier partial inventory, including two runbook-only branches marked for later cleanup and several deferrals. Those two branches are no longer present in the current `git branch`/`git worktree list` inventory.
- Open PR status was not checked in this documentation task. The branch/worktree inventory and stash contents should be revalidated when implementation begins.
