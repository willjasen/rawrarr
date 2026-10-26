# Encrypted config peer-repair protocol

## Status

Approved design for the initial two-fossil implementation. The code now has a ciphertext-only local Valkey mirror, signed durable revision records, challenge-authenticated fixed-object peer RPC, and periodic manifest/Merkle reconciliation. The isolated end-to-end reconnect gate is still pending. At the initial inspection, the MacBook Valkey service cached encrypted artwork only, the Mac mini fossil had no Valkey service, the MacBook had `/shared/config.aged`, and the Mac mini did not. See [issue #100](https://github.com/willjasen/rawrarr/issues/100) and the [task runbook](../runbooks/2026-10-26-valkey-metadata-mirror-rpc.md).

## Scope and data boundary

- Replicate only the encrypted `/shared/config.aged` object and its signed, versioned revision/tombstone records through peer RPC and Merkle reconciliation.
- Keep app metadata and artwork under `/shared/metadata` on their existing Syncthing path. They are outside the Valkey mirror, RPC manifest, and Merkle tree for this phase.
- Keep `/shared/config.aged` as the path consumed by the application, but exclude it from Syncthing on each fossil. Add the root-level pattern `/config.aged` to a local `.stignore` in each Syncthing folder. Syncthing does not propagate `.stignore`; both nodes require separate configuration. Make this change only after both fossils run the RPC-capable published release.
- Store and transmit exact age ciphertext bytes. Never decrypt configuration in Valkey, the RPC service, coordinator, or logs. Configuration may contain API credentials and metadata filename keys.
- A local Valkey entry is a disposable mirror. `/shared/config.aged` is the active local application file; an authorized peer can restore it if it is missing. If the active file, local mirror, and every peer copy are lost, Rawrarr cannot reconstruct the ciphertext or its secrets.

## Valkey representation and rebuild

Use an immutable Valkey key per config ciphertext digest, scoped by the pinned instance-root fingerprint:

`rawrarr:config:v1:<root-fingerprint>:<sha256-ciphertext>` → exact `config.aged` bytes

Keep a separate local index identifying the current revision. Do not store plaintext fields, decoded JSON, age identities, recipient private keys, or API secrets. A Valkey miss, eviction, digest mismatch, or outage falls back to the local file; a valid local file repopulates Valkey best-effort. On startup, hash `/shared/config.aged` and rebuild the mirror entry. The Valkey mirror must report whether the current ciphertext is present and digest-verified.

On peer repair, stream the ciphertext to a node-local staging file, enforce the 2 MiB config limit while reading, verify the declared byte length, SHA-256 digest, age ciphertext prefix, revision signature, instance root, and device certificate, then atomically rename into `/shared/config.aged`. Populate Valkey only after the file is durable. Retries for the same digest are idempotent. Do not expose a generic filesystem path in RPC; the object type is fixed to `config.aged`.

## Revisions, offline writes, and conflicts

Each config write creates an immutable signed revision record in node-local durable `/data/config-revisions/`; it is transferred through RPC and mirrored in Valkey but is not placed in `/shared/metadata`. The signed canonical payload binds protocol/schema version, the pinned instance-root fingerprint, stable author device ID, per-device logical counter, operation (`put` or `delete`), and ciphertext digest (for `put`). It includes no config plaintext and no host timestamp.

Before signing a local revision, advance a counter persisted atomically under `/data` above the highest verified counter known to that node. Offline writes remain allowed. Preserve every signed revision and tombstone in the initial two-node phase. Concurrent revisions for the same config converge on the lexicographically greatest `(counter, stable device ID, ciphertext digest)` tuple; host clocks do not select a winner. A delete is a signed tombstone and remains until each currently authorized peer acknowledges it. Since device revocation is not implemented, do not compact tombstones automatically.

On first enrollment into this protocol, the existing MacBook config file is imported as a baseline revision without rewriting its ciphertext. The Mac mini obtains and verifies that revision and exact ciphertext before it begins local config writes. A malformed existing config, counter rollback, incompatible schema, or unknown root fails closed and retains the prior file for recovery.

This detects many partial rollbacks when a peer retains a newer signed revision. It cannot prove continuity if every copy of both `/shared` and node-local `/data` is restored from the same older snapshot. Record this limit; do not claim hardware-backed anti-rollback.

## Peer authorization and RPC

Use an explicit, owner-managed allowlist of the two peer device IDs and direct Tailscale HTTPS URLs for the first deployment. A peer presents its existing instance-root-signed device certificate. The receiver verifies the pinned root fingerprint, certified device ID/role, expected allowlist entry, and certificate signature. No shared peer secret is introduced. A later coordinator-backed peer discovery endpoint is a separate change.

Configure the same two-entry `RAWARR_CONFIG_PEERS` JSON object on each fossil. Keys must be the stable node IDs; values must be HTTPS URLs on the tailnet ending at each node's direct API base, such as `https://node-name.example.ts.net/direct-api`. The service rejects non-Tailscale hostnames, credentials, query strings, and fragments. Do not put API keys or age identities in this setting.

Before transfer, peers use a protocol-versioned challenge exchange with a fresh 256-bit receiver nonce. The sender signs a canonical proof binding the nonce, sender ID, receiver ID, protocol version, and requested capability. Challenges expire after 30 seconds and are one-use. Each request signs the method, exact path, body digest, sender ID, receiver ID, and a fresh nonce. Reject unknown peers, invalid signatures, expired/reused challenges, unsupported versions, and mismatched targets.

The protocol exposes only:

1. `hello`: challenge, identity/capability exchange, and config revision Merkle root;
2. `manifest-page`: bounded revision leaves for a requested range;
3. `get-config`: fetch ciphertext by verified revision digest;
4. `get-revision`: fetch one signed config revision or tombstone;
5. `acknowledge`: record a verified revision generation.

Limit a manifest page to 128 revision leaves, config ciphertext to 2 MiB, concurrent transfers per peer, total staged bytes, and request rate. Route through Tailscale where available, but require device-key authentication at the application layer. Never allow callers to choose a filesystem path or fetch `/shared/metadata` through this API.

The coordinator may assist discovery later but receives no configuration ciphertext, plaintext, age identity, or signing seed. The prototype has no primary lease and no coordinator-held write authority.

## Merkle inventory

The Merkle tree covers signed config revision records only. Sort leaves by revision ID (SHA-256 of the canonical signed payload plus signature). Canonically encode each leaf and hash with SHA-256 domain separation:

- leaf payload: schema version, revision ID, device ID, logical counter, operation, and ciphertext digest;
- leaf hash: `SHA256(0x00 || canonical_leaf)`;
- parent hash: `SHA256(0x01 || left_hash || right_hash)`;
- an unpaired final child is carried unchanged to the next level;
- empty root: `SHA256("rawrarr-config-merkle-empty-v1\\0")`.

Merkle equality is an integrity/reconciliation hint, not authorization. Verify peer certificates, revision signatures, authorization, schema, ciphertext length, digest, and age format independently before accepting data.

## Two-node verification gate

An isolated integration must use two fossil services with independent `/data`, `/shared`, and Valkey instances. It must verify:

- initial import and exact repair from the MacBook-present/Mac-mini-missing state;
- Valkey loss and rebuild from local `config.aged`;
- missing local file restored from the authorized peer, then mirrored to Valkey;
- deterministic Merkle roots and concurrent offline config edits followed by reconnect;
- idempotent retries and recovery from interrupted transfer;
- rejection of unknown certificates, invalid signatures, replay, stale challenges, unsupported schema, oversized data, and arbitrary object paths;
- tombstone retention while a peer is offline;
- no config plaintext or API credentials in Valkey, peer payloads, coordinator observations, or logs;
- `.stignore` excludes exactly root `/config.aged` on each device while app metadata and artwork continue syncing through Syncthing.

Passing this gate demonstrates config convergence and repair for this two-node topology only. It does not demonstrate automatic failover, RF=3, or recovery when all ciphertext and replicas are lost.

## Review gate

Before durable RPC routes or live Syncthing exclusions are implemented, review the fixed-object API, per-device peer allowlist, counter/tombstone rules, 2 MiB limit, challenge framing, and two-node test gate. Deploy the exclusion only after both services run code that can repair config over RPC. Verify each restored ciphertext can be decrypted by the intended owner flow; a byte-for-byte hash check alone proves transfer integrity, not decryptability.
