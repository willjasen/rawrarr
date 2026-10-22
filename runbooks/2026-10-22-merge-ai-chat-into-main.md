# Merge AI chat work into main

## Objective and initial state

Merge the Rawrarr AI chat, voice, and tool-support work from `codex/ai-request-inbox` into the current `main`. At the start, the AI branch was clean at `73eeaed`; `main` was at `3d3c177` and had unrelated uncommitted coordinator and runbook edits in its primary checkout. The branches diverged substantially, so a read-only merge-tree preview was used before integration.

## Actions and findings

- The branch contains the first OpenAI chat and press-to-talk voice path, trial daily limit, Movies-versus-TV answer, Chat/rawrs tabs, SSE text streaming, account-specific local test quota override, and recorded tool-support direction.
- A merge preview reported content conflicts in `ROADMAP.md`, `runbooks/README.md`, `rust-arr/src/coordinator.rs`, and `scripts/tests/README.md`. The main checkout also had concurrent edits in some of these paths. Resolved the conflicts in an isolated worktree based on current `main`, then integrated without including the existing dirty changes.
- Resolved the four conflicts by retaining the newer main roadmap and runbook entries, combining the coordinator's node-rejection and AI routes, and keeping current subscriber tests while adding the AI chat test entry.
- Main already has `0006_revoked_nodes.sql`; renamed the incoming AI request migration to `0007_ai_requests.sql` to preserve unique ordered migration versions.
- Kept the local test-only unlimited quota override scoped to the owner's account and outside Git. It is not part of the merge.

## Verification and final state

Fast-forwarded `main` to merge commit `ae004df` (`Merge AI chat, voice, and review inbox`). Renumbered the incoming AI schema migration to `0007_ai_requests.sql` because current `main` already uses migration `0006` for revoked nodes. Rust `cargo check --locked --offline` passed with the existing `read_shared_recipient` dead-code warning; the default host Rust launcher failed on its known missing Z3 dynamic library, so the explicit Rust 1.85.1 compiler was used. No application tests or live deployment were run for this merge. The main checkout's pre-existing dirty changes were stashed temporarily, restored after the fast-forward, and remain unstaged and uncommitted in `runbooks/README.md`, `rust-arr/src/coordinator-workspace.mjs`, `rust-arr/src/coordinator.rs`, `scripts/tests/test_rawrarr_coordinator_workspace.mjs`, and `runbooks/2026-10-21-restore-droplet-arr-access.md`.

## Recovery and remaining work

The feature branch and isolated merge branch remain available as recovery sources. The merge adds only the AI branch commits; the restored uncommitted files listed above are excluded. Push `main` after confirming the configured upstream and outgoing commit list.
