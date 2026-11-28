# Fix coordinator migration failure during deployment

Issue: [#535](https://github.com/willjasen/rawrarr/issues/535)

## Objective and initial state

The user reported a published image build succeeding while the internal coordinator updater fails at SQLx `VersionMissing(13)` and rolls back before public promotion. Public was reported healthy on revision `de33b09`, without the wallet-link flash fix. The main checkout was clean and tracking `origin/main`; work is isolated in `codex/fix-coordinator-migration-collision` from freshly fetched `origin/main` (`20bf364`).

Relevant operational context: [coordinator auto-updater installation](2026-11-03-deploy-coordinator-auto-updater.md) and [container deployment guidance](../docs/container-deployment.md).

## Investigation and actions

- Confirmed the source had two migrations numbered `0013`: `app_key_grants` and `wallet_first_account`. Kept the already published app-key grants migration at version 13 and moved wallet-first signup to version 14.
- Configured the coordinator SQLx migrator to ignore newer recorded migration versions. This lets an older image start during rollback after an additive migration was recorded by a failed forward deployment. Forward schema changes still need to remain compatible with the supported rollback window.
- Added a migration-version regression test and an updater regression proving that an internal deployment error stops before the public coordinator deployment.
- The auto-updater already deploys internal before public and propagates internal deployment/verification errors; no production database or updater logic was changed.
- Rebased the task branch onto `origin/main` at `0b0cc75` after three unrelated test-gating commits arrived; reran the focused Python and Node checks successfully.
- Read the existing updater installation runbook and deployment guidance before making changes.

## Verification and final state

- `python3 scripts/tests/test_sqlx_migration_versions.py` — passed (2 checks for unique ordered versions and rollback-compatible migrator configuration).
- `python3 scripts/tests/test_do3_coordinator_auto_updater.py` — passed (20 tests, including no public promotion after an internal failure).
- `python3 -m py_compile scripts/tests/test_sqlx_migration_versions.py scripts/tests/test_do3_coordinator_auto_updater.py` — passed.
- `node --test scripts/tests/test_rawrarr_passkey_ui.mjs` — passed (7 tests).
- `node --test scripts/tests/test_rawrarr_docker_healthchecks.mjs` — passed (6 tests).
- `cargo check --locked --offline --manifest-path /workspace/rust-arr/Cargo.toml` — passed in a uniquely named OrbStack test clone after copying current worktree source, preparing dependencies, and disconnecting the clone from the bridge; only existing dead-code warnings were reported.
- `git diff --check` — passed.
- Read-only do3 inspection succeeded through strict OpenSSH with a PTY after the Tailscale SSH wrapper could not open the remote terminal. The `ai` tmux session was left running.
- Internal `rawrarr-int-roarlay` was restarting/unhealthy on revision `de33b099d4324f7e022521350fe807ec324a6eb8`; logs repeatedly reported `VersionMissing(13)`. PostgreSQL's `_sqlx_migrations` ledger showed versions 1–13 all marked successful.
- Public `rawrarr-prod-roarlay` was healthy on the same `de33b099d4324f7e022521350fe807ec324a6eb8` revision. The host's local amd64 base image was revision `0b0cc7594351aed3666abac312b3368ba69e4dd2`.
- The auto-update timer was enabled and waiting to retry. Its prior service run failed internal verification and restored the saved image. Neither coordinator deployment lock existed at inspection time.
- No production database, container, timer, or deployment files were changed.

## Adoption, recovery, and remaining work

The fix adopts through the normal published application-image path. Migration 14 runs additively after existing migration 13. If a failed deployment records a newer migration, the previous image can start while ignoring that unknown version; preserve the database and roll forward when possible. Do not delete migration rows or manually alter production schema to recover. The public coordinator remains unchanged until the internal image is healthy and verified. Deployment is pending a reviewed merge and a subsequent published-image update. The live source of the rollback failure is verified: the restored `de33b099` binary lacks migration 13 while the database has successfully applied it.
