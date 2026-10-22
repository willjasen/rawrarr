# Merge AI chat work into main

## Objective and initial state

Merge the Rawrarr AI chat, voice, and tool-support work from `codex/ai-request-inbox` into the current `main`. At the start, the AI branch was clean at `73eeaed`; `main` was at `3d3c177` and had unrelated uncommitted coordinator and runbook edits in its primary checkout. The branches diverged substantially, so a read-only merge-tree preview was used before integration.

## Actions and findings

- The branch contains the first OpenAI chat and press-to-talk voice path, trial daily limit, Movies-versus-TV answer, Chat/rawrs tabs, SSE text streaming, account-specific local test quota override, and recorded tool-support direction.
- A merge preview reports content conflicts in `ROADMAP.md`, `runbooks/README.md`, `rust-arr/src/coordinator.rs`, and `scripts/tests/README.md`; the main checkout also has concurrent edits in some of these paths. Resolve in an isolated worktree based on current `main`, then integrate that result without including the existing dirty changes.
- Keep the current local test-only unlimited quota override scoped to the owner's account and outside Git. It is not part of the branch diff.

## Verification and final state

Pending isolated merge resolution and integration. Do not run the application tests unless the user asks; verify the merge diff and Git state, and report any behavior that remains unverified.

## Recovery and remaining work

The feature branch remains available as the recovery source. Preserve and restore every pre-existing main-worktree change after advancing `main`. Push only after confirming that the upstream is configured and the outgoing merge contains only this requested AI branch work.
