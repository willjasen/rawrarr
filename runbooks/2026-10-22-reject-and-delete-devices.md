# Reject and delete coordinator devices

## Objective and initial state

Add an owner action to reject pending enrollment requests and a separate action to remove approved devices from the internal coordinator. Work is in the `codex/node-setup-collapse` worktree, initially clean at `1dc8ba1`. Existing deployment notes are in [the internal coordinator deployment runbook](2026-10-22-deploy-internal-mini-coordinator.md) and [the pending approval fix](2026-10-22-fix-node-approval-reuse.md).

## Changes

- Added owner-scoped Reject and Delete actions to the coordinator Devices view.
- Rejection deletes only an unapproved enrollment request and returns not found for an approved or foreign request.
- Device deletion revokes the approved bearer credential, removes its sessions and enrollment metadata, clears in-memory authorization, and records a tombstone so a preconfigured registry entry is not restored on coordinator restart. Device-local data is unchanged.
- Added migration `0006_revoked_nodes.sql` and focused source checks.

## Verification and deployment

- `node --test scripts/tests/test_rawrarr_node_approval_ui.mjs`: all eight checks passed, including the owner-scoped Reject and Delete source checks.
- `git diff --check` passed. Local Rust compilation could not start because the host's Homebrew LLVM dependency cannot load `libz3.5.1.dylib`; the internal coordinator Docker release build compiled successfully with Rust 1.85.
- Committed implementation and focused checks as `a8416e2` (`Add coordinator device rejection and deletion`).
- Transferred only the coordinator source files and migration to the Mac mini, preserving source backups under `~/Library/Application Support/rawrarr/internal-coordinator/backups/pre-a8416e2/`.
- Rebuilt only `roarlay` in `rawrarr-internal-mini`; PostgreSQL and the dedicated Tailscale sidecar remained running. Migration version 6 is present, roarlay is healthy with zero restarts, and local and tailnet `/health` checks returned HTTP 200.
- Read back Tailscale Serve JSON: `rawrarr.risk-mermaid.ts.net:443` `/` still proxies to `http://127.0.0.1:8080`.
- Opened a new authenticated Chrome tab at `/#devices`. It showed Reject beside Approve and Delete device on the enrolled node. Neither action was clicked; the existing pending request and device were left unchanged.
- Released and confirmed removal of the local roarlay deployment marker after verification.

## Recovery

Revert the task-owned commit and rebuild the internal coordinator. The migration is additive; the tombstone table can remain in place after a code rollback without affecting existing coordinator behavior.
