# Encrypted fossil peer-repair protocol

## Status

The initial config repair is deployed in code; this change adds fixed-path encrypted metadata RPC behind explicit `/evt` write mode. A code-level two-node RPC integration now exercises each direction using isolated node processes and authorized mock peers, including stale reconnect repair, ciphertext transfer, conflict resolution, tombstones, and idempotent retry. The broader service-topology deployment gate below remains necessary before enabling event mode on real nodes. The legacy write mode remains the default during mixed-version rollout. See [issue #100](https://github.com/willjasen/rawrarr/issues/100), [issue #486](https://github.com/willjasen/rawrarr/issues/486), and the [task runbook](../runbooks/2026-10-26-valkey-metadata-mirror-rpc.md).

## Scope and data boundary

- In legacy mode, keep config and metadata under `/shared` while compatible releases roll out.
- In event mode (`RAWARR_EVT_WRITE_MODE=evt`), place config at `/evt/fossil/config.aged` and encrypted metadata at `/evt/fossil/metadata`; exchange metadata objects and signed revisions with authorized fossil peers over RPC. Never mount `/evt/fossil` in Syncthing.
- Config RPC and its current Syncthing exclusion remain a separate existing setup. Add the root-level pattern `/config.aged` to `.stignore` on each fossil only after all participating nodes run the RPC-capable release.
- `/evt/boneyard` remains a separate Syncthing folder for explicitly selected files and media.
- Store and transmit exact age ciphertext bytes. Never decrypt configuration in Valkey, the RPC service, coordinator, or logs. Configuration may contain API credentials and metadata filename keys.
- A local Valkey entry is a disposable config mirror. The local event file is authoritative in event mode; an authorized peer can restore it if it is missing. Fossil metadata objects are retained in a node-local content-addressed cache so prior signed revisions remain transferable after a later write. If every ciphertext copy is lost, Rawrarr cannot reconstruct it or its secrets.

## Valkey representation and rebuild

Use an immutable Valkey key per config ciphertext digest, scoped by the pinned instance-root fingerprint:

`rawrarr:config:v1:<root-fingerprint>:<sha256-ciphertext>` → exact `config.aged` bytes

Keep a separate local index identifying the current revision. Do not store plaintext fields, decoded JSON, age identities, recipient private keys, or API secrets. A Valkey miss, eviction, digest mismatch, or outage falls back to the local file; a valid local file repopulates Valkey best-effort. On startup, hash `/shared/config.aged` and rebuild the mirror entry. The Valkey mirror must report whether the current ciphertext is present and digest-verified.

On peer repair, stream the ciphertext to a node-local staging file, enforce the 2 MiB config limit while reading, verify the declared byte length, SHA-256 digest, age ciphertext prefix, revision signature, instance root, and device certificate, then atomically rename into `/shared/config.aged`. Populate Valkey only after the file is durable. Retries for the same digest are idempotent. Do not expose a generic filesystem path in RPC; the object type is fixed to `config.aged`.

## Revisions, offline writes, and conflicts

Each config write creates an immutable signed revision record in node-local durable `/local/config-revisions/`; it is transferred through RPC and mirrored in Valkey. Each metadata object has a parallel signed record in `/local/fossil-revisions/`. Its canonical payload binds protocol/schema version, pinned instance-root fingerprint, fixed relative object path under `metadata/`, author device ID, logical counter, operation, and ciphertext digest (for `put`). It contains no plaintext or host timestamp. Exact encrypted objects are cached at `/local/fossil-objects/<digest>` for repair.

Before signing a local revision, advance its logical counter above the highest verified counter known to that node. Offline writes remain allowed. Preserve every signed revision and tombstone. Concurrent revisions for the same object converge on the lexicographically greatest `(counter, stable device ID, ciphertext digest)` tuple; host clocks do not select a winner. A delete is a signed tombstone. Tombstones are retained indefinitely because peer acknowledgements and device revocation/compaction are not implemented.

On first enrollment into this protocol, the existing MacBook config file is imported as a baseline revision without rewriting its ciphertext. The Mac mini obtains and verifies that revision and exact ciphertext before it begins local config writes. A malformed existing config, counter rollback, incompatible schema, or unknown root fails closed and retains the prior file for recovery.

This detects many partial rollbacks when a peer retains a newer signed revision. It cannot prove continuity if every copy of event data and node-local `/local` is restored from the same older snapshot. Do not claim hardware-backed anti-rollback.

## Peer authorization and RPC

Use an explicit, owner-managed allowlist of the two peer device IDs and direct Tailscale HTTPS URLs for the first deployment. A peer presents its existing instance-root-signed device certificate. The receiver verifies the pinned root fingerprint, certified device ID/role, expected allowlist entry, and certificate signature. No shared peer secret is introduced. A later coordinator-backed peer discovery endpoint is a separate change.

Configure `RAWARR_CONFIG_PEERS` as a JSON object keyed by stable node IDs. The
default accepts HTTPS `.ts.net` bases such as
`https://node-name.example.ts.net/direct-api`. For an opt-in Rayfish route,
set `RAWARR_CONFIG_PEER_RAYFISH_HTTP=true` and map the remote ID to its local
forward, for example `http://127.0.0.1:17443`. Only IPv4 loopback on ports
17000–17999 at the root is accepted for this form. The opt-in starts a
config-RPC-only HTTP listener on `127.0.0.1:8443` on metadata nodes. Don't
publish it or configure a non-Rayfish forward to it. Configure the receiving sidecar's
`allow_config_peer_in` for the expected Rayfish endpoint and the originating
sidecar's `config-peer` forward for the pinned receiver. Configure each node
to allow the other node ID so the existing signature checks still apply.
Rayfish encrypts the node-to-node hop; the two local hops use HTTP without
proxies or redirects. This setting does not affect coordinator HTTPS or browser routes.
Credentials, query strings, and fragments are not allowed in either route.

No persisted data or protocol migration is needed. Leave `.ts.net` routes in
place until both peers are upgraded and the Rayfish route is verified. For
rollback, restore the original peer URLs and remove the dedicated forward and
grant. Do not put API keys or age identities in the peer map or sidecar config.

Before transfer, peers use a protocol-versioned challenge exchange with a fresh 256-bit receiver nonce. The sender signs a canonical proof binding the nonce, sender ID, receiver ID, protocol version, and requested capability. Challenges expire after 30 seconds and are one-use. Each request signs the method, exact path, body digest, sender ID, receiver ID, and a fresh nonce. Reject unknown peers, invalid signatures, expired/reused challenges, unsupported versions, and mismatched targets.

The existing RPC exposes challenge-authenticated config revision pages and config ciphertext by digest. Event mode adds only:

- `GET /config-peer/fossil-manifest?offset=&limit=` — at most 128 signed metadata revision records per page.
- `GET /config-peer/fossil-object/<sha256>` — ciphertext by digest; no caller-supplied path.

Each request uses the existing one-use, 30-second challenge and device signature, and requires a metadata-role certificate from an allowlisted peer under the pinned instance root. Object paths in signed records must be relative `.aged` files below `metadata/`; symlinks and non-regular entries are rejected. The receiver verifies certificate, signature, path, operation, ciphertext prefix, 8 MiB object limit, and SHA-256 before caching or writing. Config ciphertext remains limited to 2 MiB. Transfers are sequential per peer; response bodies are streamed with a hard size cap. The coordinator receives no ciphertext, revisions, age identities, or signing seeds.

The coordinator may assist discovery later but receives no configuration ciphertext, plaintext, age identity, or signing seed. The prototype has no primary lease and no coordinator-held write authority.

## Merkle inventory

The config Merkle tree covers signed config revisions; event mode also computes a separate Merkle tree over signed fossil-object revisions. Each tree sorts leaves by revision ID (SHA-256 of the canonical signed payload plus signature). Canonically encode each leaf and hash with SHA-256 domain separation:

- leaf payload: schema version, revision ID, device ID, logical counter, operation, and ciphertext digest;
- leaf hash: `SHA256(0x00 || canonical_leaf)`;
- parent hash: `SHA256(0x01 || left_hash || right_hash)`;
- an unpaired final child is carried unchanged to the next level;
- config empty root: `SHA256("rawrarr-config-merkle-empty-v1\\0")`;
- fossil empty root: `SHA256("rawrarr-fossil-merkle-empty-v1\\0")`.

Merkle equality is an integrity/reconciliation hint, not authorization. Verify peer certificates, revision signatures, authorization, schema, ciphertext length, digest, and age format independently before accepting data.

## Two-node verification gate

The code-level two-node RPC test does not replace this service-topology integration. Before deployment, use two fossil services with independent `/local`, `/evt`, legacy `/shared`, and Valkey instances. Verify:

- initial import and exact repair from the MacBook-present/Mac-mini-missing state;
- Valkey loss and rebuild from local `config.aged`;
- missing local file restored from the authorized peer, then mirrored to Valkey;
- deterministic Merkle roots and concurrent offline config edits followed by reconnect;
- idempotent retries and recovery from interrupted transfer;
- rejection of unknown certificates, invalid signatures, replay, stale challenges, unsupported schema, oversized data, and arbitrary object paths;
- tombstone retention while a peer is offline;
- metadata/artwork object repair, digest verification, simultaneous offline edits, deterministic winner convergence, and retained delete tombstones;
- rejection of malformed metadata paths, object symlinks, unauthorized fossil certificates, oversized/chunked object responses, invalid signatures, and digest mismatches;
- that `/evt/fossil` is absent from Syncthing and selected `/evt/boneyard` content remains in the Syncthing domain;
- no config plaintext or API credentials in Valkey, peer payloads, coordinator observations, or logs;
- `.stignore` excludes exactly root `/config.aged` on each device while app metadata and artwork continue syncing through Syncthing.

The code-level test verifies the fossil-object RPC path in both directions, including reconnect, conflict, tombstone, and retry behavior; it does not exercise the config Valkey mirror, deployed service composition, or Syncthing isolation. Passing the broader service gate demonstrates config and fossil-object convergence for this two-node topology only. Neither gate demonstrates automatic failover, RF=3, or recovery when all ciphertext and replicas are lost.

## Review gate

Before enabling event write mode on nodes, deploy the compatible release to every authorized fossil, run the isolated object repair/conflict gate, verify data has copied into `/evt/fossil`, and validate owner decryption. The current release leaves generated Compose in legacy mode by default. Rollback after event-mode writes requires quiescing all writers, copying the selected winning revisions back to `/shared`, verifying digests and owner decryption, then returning every peer to legacy mode; never delete either tree automatically. This protocol does not change coordinator storage or Syncthing membership.
