# Record documentation review commits

## Objective and initial state

Make future whole-project documentation reviews aware of the previous committed review and later Git changes. The working tree was clean on `main` at `61fc927`, the documentation review commit. Related: [the documentation review](2026-10-18-current-state-documentation-review.md) and [task-owned Git workflow](2026-10-18-task-owned-git-workflow.md).

## Actions

- Added a project instruction to inspect the last recorded documentation review commit and subsequent history, then check claims against current source and relevant live evidence.
- Required a documentation commit first, followed by a separate `AGENTS.md` follow-up commit recording its hash, date, and scope.
- Recorded `61fc927` as the current review baseline and linked its runbook.

## Verification and final state

Reviewed the staged diff for task-only changes, checked its Markdown link and whitespace, and committed this instruction and runbook separately from `61fc927`. No runtime state changed. No remote or upstream is configured, so the commit remains local.

## Recovery

Revert only the follow-up instruction commit if this workflow needs revision; the documentation commit remains independent.
