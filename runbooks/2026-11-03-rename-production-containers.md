# Rename production containers

Issue: [#405](https://github.com/willjasen/rawrarr/issues/405)

## Scope and compatibility

Rename all containers defined by the public production Compose project to `rawrarr-prod-*`. The target project is `rawrarr-roarlay-prod` on do3. Keep the Compose project and service names, networks, volumes, secrets, published endpoints, and service configuration unchanged. Do not touch the internal coordinator, subscriber project, unrelated Swarmsail containers, or any public behavior.

The production base Compose file is host-local at `/opt/rawrarr-roarlay-prod/docker-compose.roarlay-prod.yml`. The repository overlay `deploy/do3/docker-compose.production-container-names.yml` supplies the container names for services across the base file and optional production overlays. Install it at `/opt/rawrarr-roarlay-prod/docker-compose.production-container-names.yml` and include it in every Compose operation that changes production services. The coordinator and support-image updaters use container names and must be updated in the same release.

Container replacement does not migrate persistent data. Keep all existing volumes and mount sources. Do not start the stopped `subscriber-provisioner`; recreate it without starting so its name and Compose labels reflect the new overlay while preserving its host-mounted data. No app image build or data/schema/protocol migration is needed.

## Published-source rollout

1. Deploy only the exact merged `main` archive. Verify its digest before and after transfer to do3. Do not build or pull an app image; this change affects deployment names and updater references only.
2. Record the active Compose files, container states and labels, image IDs, mounts/volumes, Caddy health, and Tailscale status. Preserve copies of the live production Compose files and installed coordinator updater script outside the repository.
3. Check all affected `.container-rebuild-<service>.lock` markers under `/opt/rawrarr-roarlay-prod`. Wait for any live marker; do not remove stale or other-task markers. Stop `rawrarr-coordinator-auto-update.timer` and wait for an active service run to finish.
4. Install the published container-name overlay and the published `auto_update_coordinators.py`. Validate the complete current production Compose file set plus the new overlay with `docker compose config --quiet`. Keep project name `rawrarr-roarlay-prod`, all environment files, secret paths, and host-local overlays unchanged.
5. Recreate only services that were running before deployment, one at a time under their own atomic lock, with `--no-deps --pull never` and the new overlay included. Recreate the database and standalone support services first; recreate `tailscale` before `roarlay` because the app shares its network namespace. Recreate `subscriber-provisioner` with `--no-start --no-deps --pull never`; verify it remains stopped. Do not start or recreate unrelated projects.
6. Verify all twelve expected names and Compose labels, the original project and service names, running/health status, image IDs, and original mounts. Confirm Tailscale is connected, Caddy is healthy, and Roarlay is healthy in the new Tailscale network namespace. Verify local Roarlay `/health` and the public `roarlay.rawrarr.net/health` route. The public Cloudflare edge may return HTTP 403 from do3; compare against its pre-deployment result and use local health and Caddy checks as the service-health authority when that known edge restriction applies.
7. Release each task-owned lock after that service is verified. Restart the coordinator auto-update timer and confirm a successful run against published `main`. Ensure it identifies `rawrarr-prod-roarlay` and makes no unrelated deployment.

## Rollback

If service health or routes fail, stop further recreations and restore the saved live Compose files and coordinator updater. Recreate only affected services with their original Compose file list and names, under their deployment locks. Keep all databases, state volumes, host-mounted configuration, images, and backups. Verify the original production health and routes before restarting the timer. Do not remove any volumes or prunes.

## Execution record

To be completed after deployment from the merged, published `main` revision with the source SHA/archive digest, affected services, preserved image/volume IDs, verification results, and rollback state.
