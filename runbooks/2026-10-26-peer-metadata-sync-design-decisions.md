# Peer metadata synchronization design decisions

## Objective and tracking

Record the owner's initial decisions for Rawrarr peer metadata synchronization in [issue #100](https://github.com/willjasen/rawrarr/issues/100). This is architecture planning only; no RPC or Merkle synchronization behavior is implemented.

## Decisions

- Adding or removing device membership requires explicit user authorization. Device keys cannot add, approve, or remove members.
- Each approved device signs metadata revisions. Signatures authenticate authorship but do not encrypt data or grant membership authority.
- The Merkle design covers encrypted objects and logical metadata records through a versioned mapping.
- Concurrent updates converge on the newest signed revision using a proposed deterministic logical counter, stable device ID, and ciphertext digest tie-breaker; wall-clock timestamps do not choose a winner. Counter persistence, rollback, and bounds remain to be specified.
- Retain deletion tombstones until every currently active, authorized device acknowledges. Removed devices stop counting only after a user-authorized membership change; new devices must catch up before writing.
- Authenticate RPC peers with their approved per-device signing keys, prefer Tailscale routes, and keep device-key authentication usable on other supported transports; do not introduce a shared device secret.
- An active device that remains offline can delay tombstone compaction indefinitely. Only user-authorized removal ends its acknowledgement obligation.
- Retain signed revision history; older ciphertext payloads may be compacted only under a separate retention/recovery policy.

## Remaining design questions

- Define safe ciphertext compaction while preserving signed revision history and recovery.
- Specify the durable signed history, trust-root and per-device key lifecycle, schema migration, and rollback rules before implementation.

## Verification and status

Reviewed the current shared-cluster plan and `git diff --check` passed. This decision record and the linked design remain proposals until the owner reviews the remaining protocol details. No runtime changes or tests were performed.
