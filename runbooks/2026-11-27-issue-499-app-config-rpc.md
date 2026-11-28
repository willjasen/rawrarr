# Issue #499 app configuration RPC slice

Issue: [#499](https://github.com/willjasen/rawrarr/issues/499)

## Objective and initial state

Add an app-scoped read RPC contract on the existing authorized fossil transport, as the first slice of #499. The source was fetched from `origin/main` at `3c32716` into the dedicated worktree `/Users/willjasen/AppData/GitHub/rawrarr-worktrees/issue-499-app-config-rpc` on `codex/issue-499-app-config-rpc`. Owner grant issuance (#497), delivery (#500), node verification (#494), and `/evt` support (#486) are present in this base. The live managed-app settings flow still uses group-encrypted root `config.aged`.

Related prior procedure: [app-scoped config primitive](2026-11-27-app-scoped-config-encryption.md). It is historical context and was not edited.

## Protocol slice

- Added `GET /config-peer/apps/<app-instance-id>/epochs/<key-epoch>/manifest` and `GET /config-peer/app-object/<app-instance-id>/epochs/<key-epoch>/<sha256>` to both fossil listeners. They use the existing one-use challenge and device-signature authentication.
- Installed grants now retain a private local proof containing only the owner-signed public grant metadata. The encrypted app identity envelope is omitted. Requests to app routes carry this proof; the peer verifies root, app instance, caller device, and epoch. The serving node must also have the matching installed app key epoch.
- Defined a signed app revision envelope with stable operation UUID, key epoch, fencing generation, ciphertext digest, put/delete operation, device certificate, owner grant proof, and a separate owner-root-signed writer fence binding one writer device to one generation. Added bounded local revision loading and an app-specific Merkle root. The generic metadata manifest remains separate.
- These endpoints are read-only. Existing settings writers, `config.aged`, and migration behavior are unchanged. The verifier defines the owner-fence wire contract, but an owner-side fence issuer, revision creation/import writer, and end-to-end app RPC reconciliation still need implementation before replication is active.

## Compatibility, adoption, and rollback

Older nodes continue using the group-encrypted root `config.aged`; this change does not redirect or rewrite that object. The new app routes are additive and no existing synchronized state is migrated. During rollout, legacy settings remain authoritative until every participating node can read both formats, app grants are installed, and a single-writer fencing authority and repeatable copy/verification procedure are exercised. Old writers must be stopped before cutover. Roll back before cutover by leaving legacy settings authoritative; after any future cutover, quiesce writers and verify a reverse copy or compatible reader before reverting. Never delete either copy automatically.

## Verification and remaining work

Run focused Rust checks for the grant-proof authorization and app revision contract using the documented isolated test container. Also verify missing-grant rejection, cross-app and wrong-device denial, secret-free grant proof and response, tombstones, duplicate operation IDs, and stale/conflicting fencing generations. Browser-facing settings integration and live deployment are out of scope for this protocol slice.

Focused checks in the isolated, network-disconnected `rawrarr-test-regression` clone passed:

- `cargo test --locked --offline --manifest-path /workspace/rust-arr/Cargo.toml secure_config::tests::staged_app_configuration_uses_only_its_app_identity -- --exact --nocapture` — 1 passed; verifies app-key grant install, public proof without its encrypted identity envelope, cross-app and wrong-device rejection, and app-only ciphertext round trip.
- `cargo test --locked --offline --manifest-path /workspace/rust-arr/Cargo.toml config_repair::tests::app_config_revision_binds_app_epoch_grant_fence_and_tombstone -- --exact --nocapture` — 1 passed; verifies the root-signed writer fence, app/key epoch binding, tombstone schema, and rejection of a locally altered fencing generation.
- `cargo test --locked --offline --manifest-path /workspace/rust-arr/Cargo.toml config_repair::tests::two_node_fossil_rpc_reconnects_resolves_conflicts_and_applies_tombstones -- --exact --nocapture` — 1 passed; existing metadata-fossil RPC reconnect/tombstone regression.
- `cargo check --locked --manifest-path rust-arr/Cargo.toml` — passed with existing dead-code warnings.
- `git diff --check` — passed.

The isolated test clone is retained stopped as `rawrarr-test-regression-tests-issue499-1795833148`. The focused tests validate grant proof and existing fossil transport behavior, not yet the new HTTP routes end-to-end. No runtime setting is connected to the new routes and no production data is rewritten.

## Final state

The app-scoped protocol types and read routes are implemented locally. Replication is not active because there is no writer/import path or owner-authorized fencing-token lifecycle yet. The live settings path remains on group-encrypted `config.aged`; later work must connect settings, perform staged migration, and test rollback before cutover.
