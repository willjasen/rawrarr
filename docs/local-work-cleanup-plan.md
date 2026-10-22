# Local work cleanup and integration plan

This plan preserves a pre-cleanup inventory of Rawrarr work found in local branches and worktrees on 2026-10-22. GitHub `main` is the implementation baseline. The five feature branches described below were later removed locally after explicit owner authorization; the implementation in those branches was not merged to GitHub `main`. Treat the inventory as a description of intended future work, not as available source code. See the [cleanup runbook](../runbooks/2026-10-22-cleanup-deferred-local-work.md) for the final disposition.

## Current local state

- `main` is checked out in the primary repository with 43 changed paths, including staged, unstaged, deleted, and untracked work. Treat all of it as preserved user work; do not clean or stage it wholesale.
- Two stashes remain: `stash@{0}` (`codex preserve pre-existing work before syncing origin/main`) and `stash@{1}` (`codex-preserve-before-pull-2026-10-22`). Their contents have not been reviewed in this task.
- The five feature branches in the historical inventory below have been removed locally. Their unique commits and uncommitted edits are not available in the remaining local branches or worktrees.
- The primary checkout still has unrelated local changes. Two unrelated PR worktrees and both stashes remain; do not infer their disposition from this feature-branch cleanup.
- Previously discussed runbook-only cleanup candidates and the small profile-name UI branch are absent from the current local branch/worktree inventory. Do not recreate or remove anything based only on the older inventory.

## Pre-cleanup feature inventory and future scope

| Branch | Ahead / behind at inventory time | Pre-cleanup worktree | Work described by the historical branch |
| --- | ---: | --- | --- |
| `willjasen-browser-only-private-access` | 46 / 167 | Clean | Large browser-only private access stack: independent browser client, encrypted peer access and supporting deployment/design changes. Compare with the overlapping browser-access implementation branch and choose one coherent integration path. |
| `willjasen-browser-access-implementation` | 52 / 167 | 9 modified paths | Browser identity handoff, root pinning, authenticated peer rendezvous, signed peer negotiation, read-only metadata RPC, ICE configuration, plus deployment and account-tailnet changes. Preserve and review the nine local edits before selecting commits. |
| `willjasen-do2-coordinator-migration` | 24 / 167 | 8 changed paths, including staged and untracked | Coordinator migration to the dedicated droplet, production Compose and Caddy setup, account-tailnet and enrollment changes, migration/recovery documentation. Review uncommitted migration-page, Compose, runbook, and test edits separately. |
| `willjasen-headscale-signup-integration` | 16 / 167 | Clean | Account-bound Headscale signup and subscriber tailnet client-key provisioning across Rust/API, migrations, Compose, tests, docs, and runbooks. Rebase or port only after resolving overlap with the coordinator migration stack. |
| `willjasen-rawrarr-node-rpc-prototype` | 3 / 167 | Clean | Node RPC and metadata replication prototype, persistent node state, and quorum status indicator. Decide whether this architecture remains desired; compare its node catalog assumptions with current main before integration. |

The browser-only access, browser-access implementation, Headscale signup, and coordinator migration branches overlapped in deployment, account provisioning, and browser trust code. If this work is resumed, first compare the retained design and task notes with current source, create fresh GitHub issues, and implement in small issue-scoped PRs. The old branch commits and uncommitted changes are no longer available locally.

## If deferred work is resumed

1. Revalidate current architecture, requirements, repository state, and open PRs against GitHub `main`; the counts and local changes in the table are historical.
2. Create fresh issues for the desired work and decide whether node RPC/quorum status and browser-only peer access still fit current product direction.
3. Resolve the overlapping access, account-tailnet, and coordinator-migration scope into separate, dependent implementation tasks. Use the table as a discovery aid, not as a complete specification.
4. Inspect the two remaining stashes independently before any future cleanup. They were not part of this operation.

## Acceptance criteria for resumed work

- Start from the current GitHub `main` source and validate the design against current project boundaries.
- Track new implementation in focused issues and PRs; verify accepted behavior before merging.
- Preserve the primary checkout's unrelated changes and the two unreviewed stashes unless separately authorized.

## Evidence and limitations

- Refreshed `origin/main` and branch ancestry were inspected on 2026-10-22. The inventory counts refer to the pre-cleanup revision noted in the opening paragraph and are historical.
- The five listed local feature branches/worktrees were removed with owner authorization; see the [cleanup runbook](../runbooks/2026-10-22-cleanup-deferred-local-work.md) for the final inventory and verification.
- The primary checkout still has unrelated changes, and two stashes remain unreviewed. Open PR state and stash contents should be revalidated when implementation begins.
