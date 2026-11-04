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
- Keep encrypted files in `/shared` authoritative. Valkey is disposable/rebuildable and should mirror encrypted metadata objects plus versioned logical mappings; it does not authorize writes.
- Prefer dynamically selected Valkey primary with failover, but require safe-majority authorization. A minority partition must not promote itself or become an independent writer. The current two-node MacBook/Mini plan cannot provide safe failover through loss or isolation of either voter, and is not RF=3.
- Permit approved devices to create locally signed revisions while disconnected. Those revisions remain pending peer convergence and reconcile deterministically after reconnect; quorum does not block local authoring.
- Use periodic Merkle anti-entropy and authenticated RPC repair, following Garage's repair pattern. Valkey primary/replica mirroring is an acceleration layer only; asynchronous replication can lose recent mirror writes, which must be recoverable from `/shared` and anti-entropy.

## Remaining design questions

- Define safe ciphertext compaction while preserving signed revision history and recovery.
- Specify the durable signed history, trust-root and per-device key lifecycle, schema migration, and rollback rules before implementation.
- Specify how pending local revisions are represented and reported before peer convergence, including how `/shared`/Syncthing handles concurrent revision files.
- Define when primary promotion is safe and how a third independent voter/node participates; test partition behavior without allowing dual cluster-wide primaries.

## Verification and status

Reviewed the current shared-cluster plan and `git diff --check` passed. The additional Valkey authority/mirroring, safe-majority promotion, offline signed authoring, and Garage-inspired anti-entropy choices were recorded in issue #100. The design remains a proposal; no RPC, Merkle reconciliation, Valkey metadata mirror, runtime changes, or application tests were implemented/performed.
