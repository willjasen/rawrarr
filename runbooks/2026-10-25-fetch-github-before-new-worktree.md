# Fetch GitHub before creating branches or worktrees

## Objective

Clarify that starting a task on a new branch or worktree first fetches GitHub and uses the latest fetched intended base ref.

## Initial state

The current checkout was clean on `main`, tracking `origin/main`. Other task worktrees were present and left untouched.

## Change

Updated `AGENTS.md` to require a GitHub fetch before creating a new branch or worktree, to base new work on the latest fetched intended ref, and to avoid updating or overwriting existing local branches or worktrees as part of that fetch.

## Verification

Reviewed the focused diff and confirmed the runbook index includes this entry. No tests were run because the change is documentation-only.

## Final state

Documentation change is complete. No branch or worktree was created for this documentation edit; the existing checkout and other task worktrees were preserved.

## Remaining work

None.
