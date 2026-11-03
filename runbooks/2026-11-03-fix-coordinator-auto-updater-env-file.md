# Fix coordinator auto-updater Compose environment

Issue: [#407](https://github.com/willjasen/rawrarr/issues/407)

## Scope and compatibility

The do3 coordinator auto-updater must pass each coordinator project's protected `secrets/analytics.env` file to Docker Compose. Compose uses `RAWARR_ANALYTICS_DB_PASSWORD` while validating and recreating the coordinator service. The current updater omitted this file, so an update could build the published image and then fail its preflight before either coordinator was changed.

This is deployment-tooling only. It does not change application behavior, persistent data, database schemas, synchronized files, or node/coordinator protocols. No migration is required. The env-file contents must never be logged or copied into command output.

## Deployment

1. Publish the exact merged commit to GitHub `main`.
2. Install `deploy/do3/auto_update_coordinators.py` from that published source on do3; do not copy it from a working tree.
3. Validate that Compose preflight succeeds for both projects using their existing root-only `secrets/analytics.env` files.
4. Resume `rawrarr-coordinator-auto-update.timer` and verify the updater completes the published-main rollout to both coordinators. Preserve existing project configuration, mounts, volumes, and rollback images.

## Rollback

If preflight or deployment fails, stop the timer to prevent repeated attempts. Restore the saved updater script from the previous published revision. The change does not alter application state, so no database or volume rollback is required.

## Verification

- Run `python3 scripts/tests/test_do3_coordinator_auto_updater.py`.
- Confirm Compose config validation and the scheduled updater complete without printing secret values.
- Verify both coordinators' image revision, health, and their existing routes after deployment.
