# Peer metadata synchronization design decisions

## Objective and tracking

Record the owner's initial decisions for Rawrarr peer metadata synchronization in [issue #100](https://github.com/willjasen/rawrarr/issues/100). This is architecture planning only; no RPC or Merkle synchronization behavior is implemented.

## Decisions

- Adding or removing device membership requires explicit user authorization. Device keys cannot add, approve, or remove members.
- Each approved device signs metadata revisions. Signatures authenticate authorship but do not encrypt data or grant membership authority.
- The Merkle design covers encrypted objects and logical metadata records through a versioned mapping.
- Concurrent updates converge on the newest signed revision using a deterministic logical counter, stable device ID, and ciphertext digest tie-breaker; wall-clock timestamps do not choose a winner.
- Retain deletion tombstones until every currently active, authorized device acknowledges. Removed devices stop counting only after a user-authorized membership change; new devices must catch up before writing.

## Remaining design questions

- Select the authenticated RPC identity and transport across Tailscale and non-Tailscale networks.
- Define safe full resynchronization and compaction for long-offline devices, tombstones, and losing revision payloads.
- Specify the durable signed history, trust-root and per-device key lifecycle, schema migration, and rollback rules before implementation.

## Verification and status

Reviewed the current shared-cluster plan and `git diff --check` passed. This decision record and the linked design remain proposals until the owner reviews the remaining protocol details. No runtime changes or tests were performed.
