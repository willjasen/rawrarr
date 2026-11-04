# Prepare per-app age identities

Issue [#110](https://github.com/willjasen/rawrarr/issues/110).

## Decision and scope

The owner selected an additive preparatory release and deferred activation. The
current `config.aged` v1 format, metadata writers, browser decryption paths, and
existing metadata ciphertext remain unchanged. This increment prepares stable
Radarr and Sonarr identities without claiming app-scoped metadata encryption,
membership enforcement, or revocation.

On an explicit Radarr or Sonarr settings save, a subscriber creates any missing
post-quantum app identity and a random stable instance UUID. It stores the
identity, recipient, and UUID in `/shared/app-identities.aged`, encrypted to the
current group recipient. The file is ciphertext only and is ignored by older
binaries. Existing configured apps need their settings saved once to prepare
their identities; unlock and read operations do not create shared identity
state. Clearing an app connection retains its identity for future migration.
Prowlarr is not included because this metadata store has no Prowlarr metadata.

The registry reader bounds file and plaintext size, requires age ciphertext,
validates schema, app type, UUID uniqueness, and a single valid private age
identity per entry, and checks that each identity derives its recorded public
recipient. Malformed or unreadable state fails closed and is never silently
replaced. New registry ciphertext is staged, decrypted, and checked for exact
plaintext equivalence before atomic replacement.

## Authorization, coordination, and limits

The current group-key mechanism is owner-unlock/manual installation, not an
approval-bound signed membership epoch. Any node that can currently unlock the
group identity can decrypt this preparatory registry. The coordinator receives
neither the plaintext identities nor the private IDs. Do not treat the
preparatory registry as a revocation boundary or as proof that a node is
currently approved.

Activation needs specific safeguards, not completion of all #109 scope: an
owner-authorized current set of nodes allowed to receive app identities;
delivery that does not disclose private IDs to the coordinator or
unapproved/revoked nodes; a revocation response that stops future delivery and
rotates affected app identities; and recoverable handling of interrupted
delivery or migration. The existing manual group-unlock mechanism does not
enforce that approved-node set, so it is insufficient for activation by itself.
These capabilities may reuse #109 work, but unrelated group lifecycle work is
not a blanket prerequisite.

Only one owner-selected node may save settings that create or update this
synchronized registry at a time. Wait for Syncthing to finish and verify the
same registry ciphertext is present before another node writes. The current
system has no cross-node compare-and-swap or writer-version negotiation, so
concurrent initialization is unsupported; conflicting or malformed copies must
be preserved for owner resolution, not overwritten with a newly generated key.

This preparatory step does not migrate metadata or change its encryption
recipient. Before a later activation, every participating subscriber, browser
export/decryption path, and any other reader or writer must understand app keys;
old writers must be stopped/fenced first. Re-encryption changes ciphertext
bytes. A migration must decrypt each original, validate its plaintext, encrypt
to the matching app recipient, decrypt the staged result, compare plaintext
equivalence, and retain recoverable original ciphertext until all records are
verified. Rollback after originals are removed cannot restore the old reader
state. No metadata migration or activation is part of this runbook.

## Adoption and recovery

There is no PostgreSQL or `config.aged` migration in this increment. An existing
deployment can adopt the preparatory code without changing old config or
metadata behavior. After updating a subscriber, unlock the existing group
identity and re-save the current Radarr/Sonarr settings on one selected node at
a time. Confirm the settings remain readable and Syncthing has replicated
`app-identities.aged` before another node performs a settings save.

Keep a recoverable copy of the encrypted registry along with the shared-folder
backup. If the file is absent, re-saving settings can generate keys only when
no identity has yet been used; if it is malformed or there is uncertainty about
which identity was previously prepared, stop and restore/resolve the existing
registry. A public recipient cannot recover its private identity. Do not delete
or regenerate a prepared identity to repair a migration or node move.

Older binaries continue to read and write `config.aged` v1 and group-recipient
metadata while ignoring the new registry. The registry remains available for a
future versioned `config.aged` cutover; it does not make old binaries capable of
reading app-recipient metadata. Deployment and activation were not performed.

## Verification

- `cargo test --locked --offline --manifest-path /workspace/rust-arr/Cargo.toml prepared -- --nocapture`
  in an isolated AMD64 test clone — 2 passed. Covers generated distinct app
  identities, stable IDs, group-only registry decryption, unchanged v1 config
  and existing metadata, and malformed-registry preservation.
- `cargo test --locked --offline --manifest-path /workspace/rust-arr/Cargo.toml secure_config::tests::encrypted_config_and_artwork_are_shareable_age_ciphertexts -- --exact`
  — passed.
- `cargo test --locked --offline --manifest-path /workspace/rust-arr/Cargo.toml secure_config::tests::legacy_recipient_mismatch_migrates_only_when_user_identity_opens_config -- --exact`
  — passed.
- `cargo fmt --manifest-path /workspace/rust-arr/Cargo.toml -- --check` was
  attempted in the isolated image but could not run because `rustfmt` is not
  installed. The host `rustfmt` binary also cannot load its missing Z3 library.
- Full Node.js, Playwright, Python integration, live-node synchronization, and
  deployment checks were not run.
