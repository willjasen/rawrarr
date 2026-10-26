# Per-instance device signing

> Identity ownership and the boundary between age confidentiality, passkey authentication, and ML-DSA signatures are summarized in [identity domains](identity-domains.md).

## Trust model

Rawrarr uses ML-DSA-65 for an owner-held per-instance root and node-local device keys. The root authorizes a subscriber's exact enrollment request, key, display name, and role. The coordinator stores only the root public key and signed approvals; the root private key is decrypted and used in the browser for an approval, then cleared from the form and active tab. It is never sent to Roarlay.

The owner must retain both the age identity and the age-encrypted root backup, plus the independently recorded root fingerprint. Losing either the age identity or encrypted root backup can prevent future approvals and make the instance difficult or impossible to recover. Root rotation and recovery are not implemented. Roarlay serves the UI code that handles the temporary plaintext, so a compromised Roarlay-delivered client could capture it; the server-side protocol does not remove that browser trust boundary.

The root fingerprint must be checked through an independent channel during node setup. A value returned only by the coordinator is not independent verification. The subscriber receives the pinned fingerprint through `RAWARR_INSTANCE_ROOT_FINGERPRINT`, validates the coordinator's published root key against it, and checks the root-signed device certificate before accepting it.

## Implemented protocol

- Each subscriber keeps its ML-DSA-65 seed in the persistent node-local `/data` directory. It is created once, reused after restart, and an invalid existing seed fails closed without replacement.
- Enrollment includes the public key and a proof-of-possession signature. The browser signs an exact, versioned approval payload containing the request ID, stable node ID, role, display name, and public key. The coordinator checks request ownership/expiry, payload equality, root fingerprint and signature, then pins the first valid root to the owner and instance database.
- Requests from an enrolled node bind the node ID, HTTP method, path, body hash, timestamp, and random nonce. The coordinator verifies the node key, enforces a short time window, and persists nonce use to reject replay across restarts.
- Existing bearer-token registrations remain usable as explicitly unverified legacy nodes until upgrade. When a new binary presents a token with a legacy registration, registration requires a new owner approval before that credential becomes signature-authenticated. Existing IDs, roles, tokens, user state, and volumes are preserved.
- Nodes write versioned ML-DSA-signed, compressed operational records beneath `/shared/logs/<device-id>/`. The record carries the signing public key and, after enrollment, its root-signed certificate. The verifier checks both record integrity and the certificate against the independently pinned root, including the certified node ID and role. Unsigned historical records remain readable as legacy files but cannot pass trusted verification.
- Device signing helpers and root approval UI keep plaintext age/root material in active-tab memory only. The age identity is not restored from browser storage; old remembered age-ID values are cleared. Approval inputs are cleared on cancel and page exit.

## Deployed-node migration

The database migration `0008_instance_device_signing.sql` is additive. It adds nullable signing fields and enrollment proof fields, creates a singleton public-root table and a replay-nonce table, and leaves existing node records untouched. A null node signing key means token-only legacy authorization; it must not be shown as signature-verified.

Migration sequence for an existing instance:

1. Back up the coordinator database and node-local `/data` volumes using the existing deployment procedure. Do not replace IDs, bearer tokens, passkeys, or shared/media data.
2. Upgrade the coordinator so migration 0008 is applied. The first owner-signed approval establishes the one-time instance root public key. Keep the age identity and encrypted root backup outside the deployed service.
3. On a trusted owner browser, unlock/import the age identity, enter the encrypted root backup and independently recorded fingerprint when approving, and approve the existing pending node key. The server attaches the approved key to the existing token identity; it does not create a duplicate node.
4. Regenerate subscriber Compose with the same node ID, role, token file, persistent `/data`, and independently verified `RAWARR_INSTANCE_ROOT_FINGERPRINT`. Recreate one subscriber at a time. Its new binary signs coordinator requests and fetches its root-signed certificate. Confirm it registers before moving to the next node.
5. During a mixed-version period, old binaries can continue token-only coordination. Treat them as unverified until they are upgraded and explicitly approved. Keep backups until all nodes register with signatures.

Rollback: restore the coordinator database backup and prior image/configuration together. The migration is additive, but rolling back coordinator code after signing fields have been populated can temporarily disable signed-node authorization; do not delete signing rows or regenerate node tokens to work around that mismatch.

## Current limits

- The root public key and approved keys/certificates are held in the coordinator database and delivered through authenticated coordinator APIs. They are not yet replicated as a public-key registry inside `config.aged`; config serializers currently preserve only their known fields.
- Signed `/shared` support currently covers allowlisted operational log records, not all metadata sidecars, manifests, encrypted config revisions, or arbitrary application files. A trusted verifier API and tests exist, but routine UI presentation or background verification of every synchronized log file is not wired up.
- This release does not implement root rotation/recovery, node revocation, stale-record rollback detection, multi-owner approval, or a general role/capability set. Roles remain the existing `metadata` and `storage` values.
- The RustCrypto ML-DSA implementation and protocol have not received an independent cryptographic/security audit. Browser and Rust code use a shared canonical JSON profile, but published cross-language known-answer vectors and comprehensive adversarial tests remain follow-up work.
- The currently compiled Rust code still allows legacy token-only nodes by design. Operators must distinguish them from signature-verified nodes and complete the documented rollout before claiming every node is cryptographically authenticated.

## Cryptographic profile

The algorithm identifier is `ML-DSA-65`. Root fingerprints are lower-case SHA-256 hex over `rawrarr-instance-root-v1\0 || public_key`. Signed objects use compact recursively key-sorted JSON as implemented by the browser helper and Rust `device_signing::canonical_json`. Protocol payloads carry domain strings so enrollment proofs, approvals, coordination requests, and records cannot be interchanged. Keep encodings and protocol versions explicit if adding another algorithm; never silently downgrade.

Rawrarr's global release-publisher identity remains a separate trust domain. A publisher key may attest official software but does not authorize an instance's nodes or their shared records.
