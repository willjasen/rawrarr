# Reject and delete coordinator devices

## Objective and initial state

Add an owner action to reject pending enrollment requests and a separate action to remove approved devices from the internal coordinator. Work is in the `codex/node-setup-collapse` worktree, initially clean at `1dc8ba1`. Existing deployment notes are in [the internal coordinator deployment runbook](2026-10-22-deploy-internal-mini-coordinator.md) and [the pending approval fix](2026-10-22-fix-node-approval-reuse.md).

## Changes

- Added owner-scoped Reject and Delete actions to the coordinator Devices view.
- Rejection deletes only an unapproved enrollment request and returns not found for an approved or foreign request.
- Device deletion revokes the approved bearer credential, removes its sessions and enrollment metadata, clears in-memory authorization, and records a tombstone so a preconfigured registry entry is not restored on coordinator restart. Device-local data is unchanged.
- Added migration `0006_revoked_nodes.sql` and focused source checks.

## Verification and deployment

Pending. Record focused checks, commit, deployment health and HTTPS route verification here before completion.

## Recovery

Revert the task-owned commit and rebuild the internal coordinator. The migration is additive; the tombstone table can remain in place after a code rollback without affecting existing coordinator behavior.
