# Implement per-instance device signing

## Objective

Add per-instance ML-DSA trust rooted in owner-held key material, preserve current token enrollment during migration, and document rollout for the existing deployed nodes.

## Initial state

Current node coordination uses node-local random bearer tokens with coordinator-stored SHA-256 verifiers and owner approval. Age identity unlock is already available in the coordinator browser workspace for local decryption. Existing repository changes are extensive and predate this task; keep unrelated files and edits intact.

## User-selected flow

The owner uses the Roarlay browser UI. A per-instance ML-DSA root key is generated client-side and backed up as age-encrypted ciphertext the owner copies and saves. When approving a node, the owner pastes that ciphertext; browser code decrypts and signs locally, and only the signed approval leaves the browser. The owner was informed that Roarlay-served code could capture input if Roarlay is compromised and chose this UI. Prominently warn that both the encrypted backup and age identity must be retained.

The owner also selected independent fingerprint verification for initial trust. A fingerprint obtained solely from Roarlay over HTTPS is not enough to establish independent trust; joining and existing nodes need a separate pin/compare procedure.

## Migration requirements

Existing token-enrolled nodes must continue to authenticate during rollout. Additive DB/config migrations must preserve node IDs, roles, passkeys, sessions where safe, Syncthing state, and volumes. Do not deploy until code is implemented and reviewed; deployments must follow the per-container lock and health/route procedure in AGENTS.md.

## Verification and final state

Record focused browser/crypto/backend checks, migration behavior, deployment/route status, and any incomplete trust or signed-record behavior here as work proceeds.

## Progress: owner-side cryptographic primitive

Added a browser-compatible client helper using the existing age JavaScript package and an explicit `@noble/post-quantum` dependency. It generates ML-DSA-65 root material, encrypts a backup to the age recipient, validates pasted backup key/fingerprint correspondence, and signs canonical approval payloads without sending decrypted root bytes. Focused round-trip tests pass. The helper is not yet connected to enrollment endpoints or deployed.

## Progress: age identity browser lifetime

Removed reading/writing the age private ID in localStorage and sessionStorage. On startup, the workspace clears any legacy copies, keeps the identity only in a module variable while unlocked, clears it on explicit clear/sign-out/pagehide, and no longer shares it between tabs. Updated browser wording and regression assertions. The focused static workspace test and ML-DSA primitive tests pass. The Sonarr Playwright test is currently blocked because its hard-coded `/usr/local/lib/node_modules/playwright-core/index.mjs` dependency is absent in this environment.

## Open implementation work

Enrollment/backend integration is still required: signed pending-request approval and server-side ML-DSA verification, candidate device key possession proof, independently pinned root fingerprint persistence on each node, a non-destructive migration for existing token-authenticated metadata/storage nodes, shared public key registry, and signed `/shared` record format/verification. ML-DSA runtime parity and crate security need validation before shipping. No deployment has been performed.
