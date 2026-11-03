# Refresh do3 support container images

Issue: [#391](https://github.com/willjasen/rawrarr/issues/391)

## Scope and compatibility

Refresh only third-party support images declared in the repository's internal and public do3 support-image Compose overlays. The updater preserves each deployment's active Compose files and service settings, pulls one service at a time, and recreates only that service. It excludes Rawrarr application images and local custom images. PostgreSQL stays on major version 17; no database schema or application protocol migration is needed. PostgreSQL data, Tailscale state, analytics state, secrets, and all other volumes remain attached.

Mutable Caddy and Tailscale tags are refreshed from their configured `2-alpine` and `stable` tags. Version- or digest-pinned images are checked against the configured image reference but are not silently moved to a different release. Changing those pins requires a separately reviewed version update. Each changed service's previous image is retained under a `rawrarr:rollback-support-*` tag. The updater restores that image if the recreated service fails readiness or the corresponding health route.

## Published-source procedure

1. Use the full SHA of a published GitHub `main` commit that contains the updater and both support-image overlays. Download its authenticated source archive to a new directory outside `/shared`, verify the archive digest, and extract it without overwriting an existing build directory.
2. On do3, confirm Docker reports `linux/amd64`, inspect free disk space, and confirm the internal and public per-service deployment markers are clear. Never prune images, volumes, BuildKit cache, or user files to make room.
3. Run `python3 <source-root>/deploy/do3/update_support_images.py --source-root <source-root>`. To target only one environment, add `--project rawrarr-internal-do3` or `--project rawrarr-roarlay-prod`.
4. The updater checks the active Compose project, service, and resolved image against the repository overlay. It waits for the relevant per-service marker, pulls only the selected service, and skips recreation if the image ID has not changed. For a changed image, it saves a rollback tag and runs only `docker compose up -d --no-deps --pull never <service>`. Locks remain held through service readiness and route verification.
5. Confirm the internal Tailscale Serve route still maps `/` on `rawrarr.risk-mermaid.ts.net:443` to `http://127.0.0.1:8080`, and that internal and public `/health` endpoints return success. Inspect the final service image IDs, Compose project labels, and health states.

If an update fails, the updater restores the prior image for that service and verifies it is running before releasing its marker. If rollback or route verification also fails, stop further changes, preserve both images and project state, and recover that service under its marker. Do not restart dependencies or delete volumes. An older image may be insufficient as rollback if a later release changes a persistent data format; this procedure limits PostgreSQL to its configured major version and does not change application schemas.

## Execution record

Pending publication of the repository updater and review of its checks. No support containers have been changed by this task at this point.
