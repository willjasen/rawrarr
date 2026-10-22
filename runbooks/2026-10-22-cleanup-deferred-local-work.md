# Clean up deferred local feature work

## Objective and initial state

Remove the five deferred feature worktrees and local branches after the owner explicitly authorized deleting their unmerged commits and uncommitted edits. The primary `main` checkout contains unrelated staged, unstaged, deleted, and untracked work and must remain untouched. Two unrelated PR worktrees and two stashes are also outside scope.

GitHub issue creation for `willjasen/rawrarr` returned API 404. Proposed issue title: “Clean up deferred local Rawrarr feature worktrees.” Proposed issue text: “Remove the five deferred local Rawrarr feature worktrees and their local branches after explicit owner authorization, including unique unmerged commits and identified uncommitted edits. Record the inventory in a dated task runbook, remove only the named local worktrees/branches, and verify those refs and paths are absent while preserving unrelated worktrees, remote branches, stashes, and the dirty primary checkout. This does not imply the implementation in those branches was merged or preserved elsewhere.”

## Scope

The current `origin/main` is `8e87883`. Branch ancestry is measured against it.

| Local branch | Ahead / behind | Worktree state |
| --- | ---: | --- |
| `willjasen-browser-only-private-access` | 46 / 169 | Clean |
| `willjasen-rawrarr-node-rpc-prototype` | 3 / 169 | Clean |
| `willjasen-headscale-signup-integration` | 16 / 169 | Clean |
| `willjasen-browser-access-implementation` | 52 / 169 | Nine modified files |
| `willjasen-do2-coordinator-migration` | 24 / 169 | Staged, unstaged, and untracked files |

The owner specifically authorized deleting these worktrees and local branches even though their changes have not been integrated. This operation does not delete remote branches.

## Actions

- Removed the five named worktrees with `git worktree remove --force` after the owner explicitly authorized discarding their unmerged commits and local edits.
- Removed the five corresponding local branches with `git branch -D`.
- Left their remote branches untouched. Left the primary `main` checkout, two unrelated PR worktrees, and both stashes untouched.

## Verification

- `git worktree list` no longer contains any of the five removed paths; it still contains the primary checkout and the two unrelated PR worktrees.
- `git branch -vv` no longer lists any of the five feature branches.
- `git stash list` still contains both original stash entries.
- `git status --short --branch` in the primary checkout still shows its pre-existing changes; no files there were staged, modified, or removed by this cleanup.

## Final state and recovery

The five feature branches and their local worktrees are removed. Their uncommitted edits and unmerged commits are no longer available from local branches. The cleanup plan remains on GitHub `main` as the record of deferred scope. The primary checkout, unrelated worktrees, and stashes remain available.
