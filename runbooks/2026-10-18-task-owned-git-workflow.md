# Task-owned Git workflow

## Objective

Align repository guidance with the new task-by-task commit workflow and reduce collisions when multiple tasks edit Rawrarr concurrently.

## Initial state

- Branch: `main` at `617994c480d0510c16184c3a96054ed22b59787f`.
- The latest commit is titled `stashing`; its body says remaining uncommitted code was stashed so AI tasks can track their work as they proceed.
- The working tree was clean, and only the primary checkout was registered as a Git worktree.
- `AGENTS.md` required saving task changes as they progressed but also said to stop if a commit would leave no uncommitted changes, which blocked task-owned incremental commits.

## Actions

- Updated `AGENTS.md` to permit focused commits for task-owned changes at meaningful milestones, including commits that leave the tree clean.
- Added guidance to inspect registered worktrees and isolate concurrent or overlapping work in a dedicated task worktree without copying or overwriting other work.
- Kept explicit-path/hunk staging, staged-diff review, and upstream-safe push boundaries.
- Added this runbook to the runbook index.

## Verification

- Confirm the edited guidance and runbook index link are present.
- Review the diff and working-tree status; do not commit in this task unless the change is clearly task-owned and the user requested the guidance change (which they did).

## Final state

Pending final diff review.

## Rollback

Revert only this task's `AGENTS.md`, runbook, and runbook-index changes if the workflow policy needs to be restored.
