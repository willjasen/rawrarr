# Rawrarr device signing design note

## Purpose

Rawrarr will eventually need a way to distinguish authorized devices from arbitrary writers in a Syncthing folder. Device signing is a candidate integrity and authorship mechanism inspired by the enrollment-root model in the sibling Tephramesh project. Syncthing peer membership alone is not sufficient application authorization.

This is a design note, not an implemented feature.

## Candidate model

- Each Rawrarr device generates its own signing key pair and binds the public key to its stable Rawrarr `device_id` during explicit enrollment.
- The private signing key remains local to that node in protected node-local storage (for Docker, likely a dedicated local volume or mounted secret). It never enters `/shared`, `config.aged`, logs, backups in plaintext, or the relay/subscription database.
- An instance enrollment root approves new device public keys. Public enrollment records, signer IDs, revocations, and signed configuration envelopes can be stored inside the age-encrypted shared configuration.
- Signatures cover canonical, versioned data including `instance_id`, `device_id`, schema/revision, and the exact payload or ciphertext hash. Receivers validate the full enrollment chain, signature, revocation status, and revision before accepting a change.
- A first device creates the root; adding another device requires a visible owner approval tied to that device's enrollment request and current config revision. Lost/replaced devices require revocation and key rotation.
- Age encryption remains responsible for confidentiality. Device signatures prove authorship/integrity; they do not grant a user access to decrypt data and do not replace passkeys or Rawrarr sharing permissions.
- Tailscale identity remains transport identity. A node's overlay membership does not automatically authorize it to sign Rawrarr configuration.

## Cryptographic choice

Tephramesh currently uses per-device P-256 ECDSA signing keys for its signed configuration envelopes and enrollment approvals. Rawrarr should reuse the architectural separation—local private key, synchronized public trust records, explicit enrollment, signed revisions—but must choose its own algorithm after a threat-model and interoperability review. Because Rawrarr has a post-quantum age requirement, do not silently copy P-256 as the final signature scheme; evaluate a post-quantum signature or a documented hybrid profile, key rotation, and long-term verification support.

## Decisions and tests before implementation

- Choose whether signatures cover only `config.aged` plaintext/envelope, all portable metadata sidecars, or both. Avoid signing unstable Syncthing transport state.
- Define genesis/root recovery, multi-owner approval, device replacement, revocation, rollback detection, and how disconnected devices learn revocations.
- Select a canonical serialization and signature algorithm with safe Rust libraries; keep `unsafe_code` forbidden in Rawrarr's own crate.
- Test tampered bytes, unknown signers, revoked keys, stale revisions, wrong `instance_id`, key rotation, restore from backup, and offline peers.
- Prove private signing keys never enter Syncthing, relay APIs, logs, process arguments, or shared backups. Encrypted backup archives may contain signed public records but never local private keys.

## Reference

The design review used Tephramesh's `readme/SIGNING.md` and `src/config-signing.ts` as an example of enrollment roots, device-bound keys, signed revisions, and revocation. Rawrarr has different runtime, storage, and post-quantum requirements, so this is a pattern reference rather than a compatible protocol.
