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
- Rebased the task branch as upstream commits arrived; the implementation was reviewed and merged through PR [#542](https://github.com/willjasen/rawrarr/pull/542) as `b99b4197b58451132832724777854b1baf5cc03c`.
- Read the existing updater installation runbook and deployment guidance before making changes.

## Verification and final state

- `python3 scripts/tests/test_sqlx_migration_versions.py` — passed (2 checks for unique ordered versions and rollback-compatible migrator configuration).
- `python3 scripts/tests/test_do3_coordinator_auto_updater.py` — passed (20 tests, including no public promotion after an internal failure).
- `python3 -m py_compile scripts/tests/test_sqlx_migration_versions.py scripts/tests/test_do3_coordinator_auto_updater.py` — passed.
- `node --test scripts/tests/test_rawrarr_passkey_ui.mjs` — passed (7 tests).
- `node --test scripts/tests/test_rawrarr_docker_healthchecks.mjs` — passed (6 tests).
- `cargo check --locked --offline --manifest-path /workspace/rust-arr/Cargo.toml` — passed in a uniquely named OrbStack test clone after copying current worktree source, preparing dependencies, and disconnecting the clone from the bridge; only existing dead-code warnings were reported.
- `git diff --check` — passed.
- Published image workflow run [46099737645](https://github.com/willjasen/rawrarr/actions/runs/46099737645) succeeded for commit `b99b4197b58451132832724777854b1baf5cc03c` on amd64 and arm64.
- Per the user's direction, deployed only the public coordinator on do3 using its existing updater and public deployment lock. The internal environment was left out of scope.
- Public `rawrarr-prod-roarlay` now reports healthy on revision `b99b4197b58451132832724777854b1baf5cc03c`; its image architecture is `linux/amd64`. The actual application database (`rawrarr_passkey_reset_20261103`) records migrations 1–14 as successful, including versions 12, 13, and 14.
- External `https://roarlay.rawrarr.net/health` returned HTTP 200 and the Roarlay health page. The updater's edge curl from do3 returned 403, but the public route check from the workstation succeeded.
- The public deployment lock was released after verification. The remote Linux work was conducted in tmux session `ai`, which was left running.
- No internal coordinator or database was changed.

## Adoption, recovery, and remaining work

The fix adopts through the normal published application-image path. Migration 14 runs additively after existing migration 13. If a failed deployment records a newer migration, the previous image can start while ignoring that unknown version; preserve the database and roll forward when possible. Do not delete migration rows or manually alter production schema to recover. The change is merged and the public coordinator is deployed and verified at `b99b419`. Internal deployment was explicitly deferred by the user and remains outside this task's completed scope.
