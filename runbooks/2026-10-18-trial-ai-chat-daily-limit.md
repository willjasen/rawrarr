# Limit trial AI chat to five daily questions

## Objective and initial state

Specify a five-use daily allowance for a trial user's AI chat. The dedicated `codex/ai-request-inbox` worktree was clean at task start. The [AI chat and voice plan](2026-10-18-plan-ai-chat-request-review.md) is documentation only; the [subscription model](../docs/subscription-model.md) plans a 24-hour instance trial but neither the AI feature nor trial enforcement is implemented.

## Decisions and actions

- Added a trial allowance of five accepted AI questions per signed-in account per UTC calendar day. Typed and voice questions share it, including authenticated phone requests when supported.
- Count a user question once by stable request ID. Internal model turns, tool calls, retries, and owner review-agent processing do not use the allowance. A request without a usable answer or accepted follow-up is refunded.
- Show remaining uses and reset time in the user's local time. An expired trial remains expired even if the daily allowance has unused questions. Paid-plan limits are undecided.
- Recorded the behavior in `docs/DESIGN.md`, `docs/subscription-model.md`, and `ROADMAP.md`.

## Verification and final state

Reviewed the changes against the existing instance trial and request-inbox design; checked links and `git diff --check`. No application code, database migration, container, or live entitlement was changed. The allowance is planned behavior only.

## Remaining work and recovery

When implementing AI chat and trial enforcement, add atomic quota reservation/refund, idempotent request IDs, clock-boundary and concurrent-request tests, and a visible allowance indicator. Revert only this task's documentation commit if the product rule changes; no runtime rollback is needed.
