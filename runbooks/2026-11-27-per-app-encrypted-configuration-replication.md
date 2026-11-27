# Per-app encrypted configuration replication direction

## Objective

Extend the general app platform direction so each app has its own age identity and can replicate app-owned configuration and other explicitly portable data as ciphertext among authorized nodes. Tracked in [#482](https://github.com/willjasen/rawrarr/issues/482) and [#110](https://github.com/willjasen/rawrarr/issues/110).

## Initial state

Continued the general app platform task in the existing `codex/general-app-platform` worktree. The worktree already contained the documentation commit `9c19397`; `origin/main` had advanced since that branch was created, so a fast-forward update of the feature worktree was unavailable. Existing #110 scoped per-app identities to metadata, and #109 defined group membership distribution/rotation.

## Decisions recorded

- Every stable app instance has a distinct age identity.
- App-owned configuration and other explicitly portable data are encrypted to that identity before replication. Replication contains ciphertext only; private app identities and plaintext stay on authorized nodes.
- App definitions classify each data item/path: portable encrypted configuration/data, node-local databases/sessions/caches/credentials, or state that requires a tested app-aware migration/backup.
- Credentials remain node-local by default. Native app databases do not become synchronized files by assumption.
- The coordinator and relay may transport ciphertext and opaque instance identifiers, but may not receive private app IDs or plaintext app data.
- Per-app key authorization, delivery, revocation/rotation, compatibility, and recovery must be defined before activating encrypted replication. #109 may provide mechanisms but is not blanket-blocking if another verified design supplies those capabilities.
- Duplicati's app configuration is the example portable encrypted data; its native database, session state, cache, and destination credentials remain local absent a tested migration contract.

## Actions

- Updated issue #482 with the per-app identity and encrypted-replication requirement.
- Expanded issue #110 to include app-owned configuration and portable data, data classification, authorized delivery, and local-only state boundaries.
- Updated `docs/product-direction.md` and `docs/DESIGN.md` to state the model and Duplicati example.

## Verification

- Ran the focused Rust tests in a uniquely named `rawrarr-test-regression` container from the current worktree snapshot. Prepared the clone while networked, disconnected it from the bridge, then ran `cargo test --locked --offline --manifest-path /workspace/rust-arr/Cargo.toml prepared -- --nocapture` — passed (2 tests).
- Coverage confirms distinct/stable prepared Radarr/Sonarr identities, group-encrypted registry storage, unchanged v1 config and existing metadata, and malformed-registry fail-closed behavior.
- `git diff --check` and review of the complete diff passed.
- These tests cover preparatory identity persistence only. They do not establish app-key authorization or encrypt/replicate app configuration using the app identities.

## Adoption and rollback

No deployment adoption or data migration is required because this increment records planned behavior only and the tested existing code remains preparatory. Revert the documentation commit and revise #482/#110 if the design changes. Activating app-recipient configuration/data replication requires a versioned encrypted-data compatibility path, old-writer fencing, authorized-node key delivery, rotation/recovery behavior, focused tests, and an app-specific adoption plan.

## Final state

The design now requires per-app age IDs for encrypted replication of explicitly portable app configuration/data, while clearly preserving local-only state and the coordinator/relay privacy boundary. Existing source prepares Radarr/Sonarr identities but does not yet encrypt configuration to them or activate app-scoped replication. No general app catalog or Duplicati support is implemented by this task.
