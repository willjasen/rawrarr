# Shared cluster plan

## Status

Planning only. No multi-person sharing, MLS protocol, revocation cleanup, or age-recipient migration is implemented by this plan.

## Product model

- Every new cluster starts **isolated** with one owner.
- The owner can make a one-way transition to **shared** by inviting another person. The cluster remains shared even after every invitee is removed. Starting over in isolated mode means creating a new cluster with a new cluster identity, age identity, and signing root; it does not erase prior access.
- The owner alone controls membership. Adding or removing a device requires an explicit user-authorized membership change; a device signing key can never grant membership or approve another device. Invitees may use the cluster but cannot invite others or approve membership changes.
- Each person authenticates with their own passkey and uses their own age identity. Private age identities, passkeys, and the owner's root signing key are never shared.
- Revocation ends future access after the group state advances. A removed person can retain any earlier data, plaintext, or keys they already obtained.
- The owner portal distinguishes **access revoked** from cleanup state: pending/waiting for the node, node-reported cleanup complete, or unconfirmed. A report is not proof of secure erasure.

## Candidate technical design

### Peer metadata synchronization with RPC and Merkle reconciliation (planned)

The long-term goal is for approved Rawrarr nodes in one cluster to exchange metadata directly, with a Garage S3-inspired model: RPC handles authenticated peer coordination and bounded data exchange, while Merkle trees summarize replicated records and identify which branches or objects differ. This is a design direction only; no peer metadata RPC or Merkle reconciliation is implemented by this proposal.

- Keep encrypted metadata objects as the replicated payload. Peers validate cluster/instance identity, membership, object class, size limits, and signed revision metadata before accepting data; a Merkle match is an integrity/reconciliation aid, not an authorization proof.
- Each approved device signs its metadata revisions. The signed envelope binds the cluster/instance, device, record class, opaque logical key, ciphertext object key and digest, schema/mapping version, and revision. The signature identifies and authenticates the author; it neither encrypts content nor changes device membership. Membership changes require a separate user-authorized record.
- Compare both encrypted object files and logical metadata records. A versioned mapping links logical records to one or more opaque ciphertext objects. Canonically encode the mapping and leaf records, sort deterministically, and build Merkle trees over both views so peers can find either missing payloads or logical references whose objects differ.
- The current logical revision is the newest signed revision under a deterministic, wall-clock-independent order. A candidate order is a persistent monotonic logical counter advanced beyond the highest observed counter, followed by stable device ID and ciphertext digest as final tie-breakers. Peers must validate counters and apply the same order; host timestamps do not decide winners. Specify counter persistence, rollback, and bounds in the protocol before adoption. Retain signed revision history; compact older ciphertext payloads only under a separately specified retention/recovery policy.
- Authenticate RPC peers with their approved per-device signing keys; prefer direct Tailscale routes when available, while keeping device-key authentication valid over other supported transports. Do not introduce a shared device secret. Specify a signed challenge/handshake, capability negotiation, accepted cluster epoch and tree-root exchange, bounded subtree comparison, and ciphertext object transfer. Coordinator/relay services may assist discovery or transport fallback but never become metadata plaintext authorities.
- Reconcile missing leaves and tombstones incrementally. Verify hashes while streaming, stage received objects, validate signatures and authorization, then atomically publish them into the local metadata store. Retries must be idempotent and safe after interruption.
- For concurrent revisions, converge on the highest revision in the deterministic order above. A device cannot authorize membership or other authority changes by signing a metadata revision; those continue to follow user-authorized trust-history rules.
- Retain each deletion tombstone until every currently active, authorized device has acknowledged it. A device removed through a user-authorized membership change no longer blocks acknowledgement, and it must pass revocation/catch-up gates before any later re-enrollment. A newly added device must receive the current tree/tombstones before it can publish writes. An active device that stays offline can therefore delay tombstone compaction indefinitely; only a user-authorized device removal ends that obligation. Do not drop tombstones on a timer and risk resurrecting deleted records.
- Persist each node's last accepted trust-history head and per-instance Merkle root locally. On rollback, stale snapshots, unknown roots, or incompatible schemas, quarantine the affected sync state and require a validated catch-up or recovery path before serving it as current.
- Keep node-local Valkey disposable and rebuildable from authoritative encrypted metadata under `/shared`. The planned mirror covers encrypted metadata objects and versioned logical mappings; it does not become the source of truth or change authorization. Merkle leaves refer to authoritative encrypted objects; Valkey residency and cache health must not affect convergence.
- If Valkey primary/replica mirroring is used as an acceleration layer, choose the writable primary dynamically but require a safe majority to authorize promotion. A disconnected minority must not promote itself. The initial plan defers a third node: with only the MacBook and Mac mini, safe automatic failover is unavailable when either node is isolated; use the pair to test convergence and reconnect repair only. Do not claim fault-tolerant automatic failover until a third independent voter is available, or three-way replication until a third data-bearing fossil is available. Valkey replication is asynchronous, so it is not itself proof that every acknowledged mirror write reached the replica.
- Allow an approved device to create locally signed revisions while disconnected. Mark them pending peer convergence; after reconnection, validate and merge them using the deterministic revision order. Quorum is not a prerequisite for local authoring. Require safe-majority authorization for Valkey-primary promotion and any operation that claims a single cluster-wide primary; a minority must not promote itself. `/shared` remains authoritative in every case, and Merkle repair must rebuild mirror state without trusting a former primary.

Implementation should begin with a protocol document and two-node sanitized prototype: canonical encoding and hash domain separation, RPC framing/authentication, pagination and resource limits, deletion/tombstone retention, schema negotiation, offline catch-up, interrupted transfers, and malicious/stale peer tests. Only after those decisions are reviewed should a durable node RPC service be implemented.

### Separate responsibilities

- **Passkeys** authenticate each person's account to Rawrarr.
- **Age** encrypts shared files directly. For each file, age generates a fresh random file key, encrypts the payload once, and puts one recipient stanza in the age file header for each authorized public age recipient. Each stanza lets its matching private age identity recover the same file key; the stanza's exact contents depend on the age recipient type and do not necessarily contain the public recipient ID verbatim. The file header therefore grows a little for every added recipient, while the encrypted payload is stored only once. Rawrarr keeps the public age recipient IDs in its authenticated membership record to determine who should receive future files. The shared folder contains the age ciphertext and its encrypted header stanzas, never the file key in plaintext and never a persistent cluster-wide pre-shared key. Each member supplies their public age recipient; their private identities remain with them. Do not add a separate cluster key or custom AES/key-wrapping layer solely to achieve one-pass file encryption.
- **Signing authority** proves which owner/device authorized membership or configuration changes. Signing keys are not age encryption keys and must not be derived from or substituted for them.
- **Membership protocol** records the authorized member set and orders transitions. MLS is a candidate because its epochs and commits describe group state changes, but it does not replace age file encryption or automatically define Rawrarr's access policy.

### Durable chain of trust

Every cluster, isolated or shared, needs a durable verifiable trust history beginning at cluster creation. The initial setup creates a unique cluster ID and a genesis record that binds the cluster ID to the initial owner/root public signing key, initial age recipient, initial device keys and roles, format versions, and initial mode (`isolated`). The owner signs this genesis record, and the owner preserves its fingerprint as the independent trust anchor.

Each subsequent authority-changing record (device add/remove, role change, owner-authorized configuration revision, isolated-to-shared transition, person invitation/removal, signing-root rotation, and recovery action) should include the cluster ID, monotonically increasing sequence or epoch, previous record hash, change type, affected public identities/roles, and the authorization proof. Nodes verify the entire chain back to their pinned genesis anchor and reject unknown roots, gaps, invalid signatures, conflicting branches, or rollback to an older head. A signature from a device key alone proves which device signed a record; it does not grant authority to change membership. The applicable owner/root authorization must also verify.

Keep signed history records append-only and replicate them across owner-controlled nodes and protected backups. Each node stores its last accepted chain head locally so replacing or rolling back `/shared` cannot silently roll back that node's trust state. A restored node whose history is behind must catch up and validate the chain before participating. Root rotation must be cross-signed by the previous trusted root and recorded as a new link; do not replace the root in-place.

“Cannot be lost” must mean the history is redundantly preserved and loss/rollback is detectable, not that cryptography can reconstruct missing records. If all copies of the history and all valid recovery authorities are lost, the original continuity cannot be proven; recovery must create a visibly new trust root/cluster or use an explicitly provisioned recovery authority. A later owner must not silently declare a new root to be the old one.

### Loss and recovery assumptions

The original media files are the primary user asset and remain unencrypted on the user's storage; Rawrarr must not delete or treat them as dependent on encrypted metadata. Age-encrypted metadata and configuration are valuable but may be rebuildable from media, configured Radarr/Sonarr sources, and fresh scans. This is not guaranteed for user-authored metadata, prior snapshots, or unavailable upstream records, so recovery screens must explain what can be regenerated and what is gone. The owner is responsible for protecting their age identity and signing key; Rawrarr does not provide a service-side secret recovery bypass.

Loss of the only age private identity that can decrypt an isolated cluster's files means its existing age-encrypted metadata/config cannot be recovered from the passkey or signing root. The owner may create a new age identity and rebuild the encrypted metadata/config from surviving media and integrations, but old ciphertext remains undecryptable and any unreproducible metadata is lost. A separately stored age-identity backup is optional owner-managed recovery if preserving old encrypted records matters; Rawrarr cannot restore the lost identity.

Loss of the signing root is different: it does not itself decrypt or erase age-encrypted metadata, but it prevents safely authorizing devices or membership changes. If the root is lost but the age identity remains, the owner can create a replacement root and explicitly re-enroll the devices, establishing a new genesis/trust chain. Devices must not trust the new root automatically, and the UI must make clear that continuity with the old signing chain has ended. This is an accepted owner-managed reset, not recovery of the old key. An owner may keep a protected backup to avoid this reset, but Rawrarr does not promise recovery. In shared clusters, another member's age identity may still decrypt content encrypted to that member, but this does not automatically restore the owner's signing authority or account ownership.

### Eventual synchronization and authority

Syncthing may carry immutable, authenticated membership protocol records and encrypted shared files. It is an eventually consistent transport, not the authority that selects the valid cluster state. To keep membership changes single-writer and rare, only an owner-authorized client may commit an invite or removal. Every node validates the owner authorization, cluster identity, prior epoch/revision, and next sequence before applying a commit. Devices with missing or conflicting history stop membership-sensitive work and reconcile before accepting new changes. Invitees cannot commit adds, even if they can construct syntactically valid MLS messages.

The plan must compare two options before protocol implementation:

1. **MLS group state:** use MLS proposals/commits, a single owner-authorized committer, explicit epoch handling, and a Syncthing-backed delivery/reconciliation layer.
2. **Owner-signed append-only membership log:** use signed, monotonic membership records with one owner writer and explicit replay/fork checks, while age recipients handle content access.

Prefer MLS only if its standardized group epoch and membership machinery provides enough value to justify per-device MLS credentials/state, offline catch-up, conflict recovery, and library integration. Neither approach should be represented as implemented until a prototype proves recovery from missed messages and concurrent stale-owner commits.

### File access and history

New shared files should be encrypted to the authorized recipients for the membership state in effect when written. On removal, future files and revisions exclude the removed member. Historical access is intentionally not revoked; copies and keys already received cannot be clawed back.

Existing Rawrarr files are generally age-encrypted to one recipient. Before promising that a new invitee receives the existing library without re-encrypting payloads, prototype a safe age header/key-wrap update with a maintained age implementation. If that cannot be supported, choose explicitly between re-encrypting historical payloads once for the new recipient set or sharing only newly created data. Never claim that adding a recipient to existing ciphertext is already supported.

## Revocation and cooperative cleanup

1. The owner removes the member and commits the next valid group state. Immediately stop issuing new file keys/recipient wraps to that member and close Rawrarr access routes.
2. Deliver an authenticated cleanup tombstone over a narrow control path or the existing sync link. Keep the tombstone available until acknowledged; it must not contain shared plaintext. Cleanup applies only to Rawrarr-managed shared content, indexes, and caches; it must not delete the member's original media or unrelated files.
3. On receiving the tombstone, the node removes its Rawrarr-managed shared data and reports completion. Rawrarr then disables the Syncthing folder/peer authorization. If there is no response by the defined timeout, the owner may disable sync immediately; retain cleanup as unconfirmed and require the node to check revocation and complete cleanup before any future re-enrollment or participation.
4. If a node is offline, show cleanup as pending. When it reconnects through any control or enrollment path, it must process the revocation and complete the cleanup/reset gate before participating in the current shared folder again.
5. Record a node's authenticated response as **reported complete**. If it ignores the request, is unreachable, or its response cannot be verified, show cleanup as unconfirmed.

The person controlling an invitee node can refuse cleanup, preserve backups, or retain copies. The owner UI must never say that revocation securely erased past data. A removed node must not be allowed to reintroduce stale shared state when it reconnects.

## Security boundaries

- Keep age private identities and per-device signing/MLS private state node-local. Do not put them in `/shared`, relay services, logs, or shared backups in plaintext.
- Keep the cluster root signing private key with the owner. It authorizes membership changes; it is not given to invitees and does not decrypt shared files.
- Make cluster mode, owner authority, membership revision/epoch, and revocation state authenticated and rollback-resistant. Define restore-from-backup behavior so an old snapshot cannot silently restore a removed member.
- Preserve an append-only, signed trust history from the genesis record across isolated and shared operation. Replicate it and pin each node's accepted head locally. Detecting loss is possible; if every history copy and recovery authority is lost, the original chain cannot be reconstructed safely.
- Treat cleanup acknowledgement as a cooperative application report, not remote attestation or proof of secure deletion.

## Phased roadmap

1. **Freeze policy and threat model.** Confirm isolated/shared semantics, owner-only membership, what history an invitee receives, what cleanup covers, age-identity and root-key loss/compromise handling, succession, and how genesis trust is independently pinned.
2. **Specify durable trust history.** Define canonical signed records, previous-hash linkage, monotonic sequence, local head pinning, replicated backups, root rotation, fork/rollback detection, and explicit recovery from lost history.
3. **Prototype transport and consistency.** Compare MLS and the signed membership-log option using two or more isolated devices over Syncthing. Test missed updates, duplicate/out-of-order files, concurrent stale commits, reconnect, conflict recovery, rollback of `/shared`, restoration from backup, and stale-node quarantine.
4. **Prove age file compatibility.** Test multi-recipient age encryption for new files and safe recipient rewrapping for existing files without changing payload bytes. Define the versioned file envelope and legacy migration before adoption.
5. **Implement owner-controlled invitations.** Add short-lived invitations, invitee-owned passkey/age identity binding, owner approval, signed membership changes, per-node state, and audited membership display.
6. **Implement revocation and cleanup reporting.** Advance the group state, stop future access, revoke sync/route authorization, deliver cleanup requests, gate reconnecting nodes, and expose pending/reported/unconfirmed status.
7. **Validate recovery and release readiness.** Exercise owner backup restore, age-identity loss and metadata rebuild, root-key loss/compromise, offline nodes, removed-member behavior, malicious non-owner proposals, old snapshots, backup retention, history gaps, and root rotation. Update security documentation and user guidance before enabling the feature.

## Open decisions

- What retention policy permits compacting losing ciphertext while retaining signed revision history and safe recovery?

- Is MLS preferable to a smaller owner-signed membership log for Rawrarr's infrequent changes?
- Does an invitation expose the existing library by default? If yes, what is the supported no-payload-re-encryption migration for existing age files?
- Is membership per person, per Rawrarr node, or per person with individually enrolled nodes? How do multi-device users remove one device versus the person?
- What explicit confirmation and device re-enrollment flow should be required when the owner resets a lost signing root and starts a new trust chain?
- Precisely which Rawrarr-managed data is deleted on a cooperating node, and how long does a pending cleanup remain actionable?

## References

- [Product and interface design](DESIGN.md) — current sharing behavior decisions.
- [Device signing design](device-signing-design.md) — candidate instance root, signing keys, enrollment, and signed records.
- [Security](SECURITY.md) — current data domains and trust boundaries.
- [RFC 9420: MLS](https://www.rfc-editor.org/rfc/rfc9420.html) and [RFC 9750: MLS Architecture](https://www.rfc-editor.org/rfc/rfc9750.html) — protocol and delivery-service behavior.
- [Age format specification](https://c2sp.org/age%40v1.0.0) — encrypted file and multi-recipient format.
