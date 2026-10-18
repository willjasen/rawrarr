# Device signing identity proposal

## Objective

Clarify the future device-signing model so nodes use unique cryptographic identities rather than host-and-role labels as their shared identity, while recording each node's function separately.

## Initial state

The repository already has a design note at [device-signing-design](../docs/device-signing-design.md). It proposes a per-device signing key and stable `device_id`, but leaves the ID derivation and role binding unspecified. Current deployment IDs include `macbook-metadata` and `mac-mini-storage`; role values are separately `metadata` and `storage`.

## Proposal recorded

Update the design note to distinguish a stable cryptographic device identity from a human-readable name and node function. Recommend a versioned public-key fingerprint as the future device ID, with an explicit role/capabilities claim in the signed enrollment record. Record metadata and storage/apex as functions, allowing one device to advertise more than one capability if needed. Existing deployment IDs and role records remain compatible until a separately planned migration.

## Boundaries

This is documentation only. No keys, IDs, enrollment protocol, configuration schema, or deployments are changed. Device-signature algorithm, canonical encoding, trust root, key rotation, revocation, and ID migration require review before implementation.

## Verification

Read the updated design note and check the runbook index entry. No tests or runtime changes are applicable.

## Follow-up: initial algorithm candidate

Brandon selected ML-DSA as the first algorithm to evaluate. The design note now names ML-DSA (NIST FIPS 204) as the initial candidate, while preserving algorithm versioning and deferring implementation pending safe Rust library, encoding, interoperability, and key lifecycle review. P-256 is only a possible separately specified hybrid component, with downgrade prevention required.

## Follow-up: coordination and shared-record trust

The design now places ML-DSA keys in two complementary trust paths. Owner-approved enrollment verifies proof of possession and binds the exact public key, derived device ID, and role. The coordinator retains approved public keys in its protected node registry for coordination authentication; the age-encrypted shared config replicates approved public verification records for peers. Nodes may sign versioned `/shared` records bound to instance, signer, role as applicable, record identity, revision, and payload/ciphertext. The config must not be its own sole trust bootstrap: reject self-introduced keys until independently approved. Private signing keys remain local.

## Follow-up: trust-root options

The design now distinguishes three instance trust choices: first device as root, a separate owner/enrollment root, or a threshold/recovery authority. Each new node must be approved by the selected authority; the first node is not automatically trusted as root without an explicit choice. It also separates a potential global Rawrarr publisher key for software/update authenticity from per-instance device authorization. No root model or global key has been selected or implemented.

## Follow-up: roarlay-held instance authority

Added a per-instance option where roarlay holds the cluster authority key created at setup and signs owner-approved enrollment/revocation records. This is distinct from a global publisher key. Documented roarlay's elevated compromise impact and the need for protected key storage plus backup, rotation, and recovery design. This remains an option, not a selected implementation.

## Follow-up: age-encrypted root backup

Recorded a candidate creation/enrollment flow: establish and back up the age identity, generate the per-instance ML-DSA root locally, and instruct the owner to copy/save its age-encrypted backup. For later approvals, the owner pastes the ciphertext; the client decrypts and signs locally after checking the pinned root fingerprint, then sends only the signed approval. Instructions must cover preserving the encrypted backup and age identity recovery. Roarlay never receives the plaintext root.

## Follow-up: explicit copy and paste instructions

Updated the proposal so the owner is explicitly instructed to copy and save the encrypted root backup at setup, then paste it into the client when approving later enrollments. The UI must explain retention and recovery requirements. Local decrypt/sign is transient; only the signed approval leaves the client.

## Follow-up: prominent loss warning

The creation/enrollment instructions now require a prominent acknowledgement that both the encrypted root backup and the age identity (with its recovery backup) must be retained. Losing the root backup prevents restoring the root; losing the age identity prevents decrypting that backup. Either can block approvals and make recovery difficult or impossible unless a distinct recovery authority is designed. Repeat this warning at backup copy, enrollment approval, and recovery guidance.
