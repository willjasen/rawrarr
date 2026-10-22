# Review remaining local Rawrarr work

## Objective and initial state

Record which local Rawrarr branches, worktrees, and stashes still contain work so it can be integrated or safely cleaned up later. The primary checkout had 43 changed paths and was eight commits behind its previous `origin/main` ref. All primary-checkout changes were treated as pre-existing and left untouched.

Related plan: [local work cleanup and integration plan](../docs/local-work-cleanup-plan.md). The historical voice inventory is available in Codex task `01a165cf-5cb6-7ca1-bf12-1e2a949c6c51`.

## Actions

- Refreshed the comparison against the available `origin/main` at `b023347` in an isolated worktree.
- Counted branch ancestry, summarized committed diffs, checked live worktree dirtiness, and listed the two remaining stashes.
- Found five outstanding feature branches: browser-only access (46 commits ahead), browser-access implementation (52 ahead plus nine modified paths), coordinator migration (24 ahead plus eight changed paths), Headscale signup (16 ahead), and node RPC prototype (three ahead).
- Identified overlap between browser access, account-tailnet provisioning, and coordinator migration; documented a grouped review followed by focused integrations.
- Confirmed the former runbook-only cleanup candidates and profile-name UI branch are absent from the current branch/worktree inventory.
- Updated `ROADMAP.md` and the runbook index to point to the durable plan.

## Verification

- Compared branch commits and file summaries with Git and checked each linked worktree's porcelain status.
- Confirmed the main checkout's 43 changed paths and both stash entries remained untouched.
- `git diff --check` passed for this task's documentation changes.
- No tests were run; this task documents deferred repository work and changes no runtime behavior.

## Final state and remaining work

No branches, worktrees, stashes, or primary-checkout changes were deleted, merged, or altered. Recheck branch ancestry, dirty files, stash contents, and open PRs before beginning integration or cleanup.
