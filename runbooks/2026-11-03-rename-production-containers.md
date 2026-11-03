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

The naming overlay was installed from published `main` revision `5624a8ef1afc325df88c36aa37119485d94608e2`; the archive SHA-256 was `c5812b432d4bdcb0edcca1830ca38d213d35e85ad4c4028581388af494c661da`. All twelve services retained the `rawrarr-roarlay-prod` project and their service labels:

| Service | Final container | State after rename | Image ID preserved through rename |
|---|---|---|---|
| `postgres` | `rawrarr-prod-postgres` | Running, healthy | `sha256:ab20e934b27ba3ff927c1b4c9ff8cf6c32f315f743efe68dd3441182a87fb085` |
| `umami-postgres` | `rawrarr-prod-umami-postgres` | Running, healthy | `sha256:ab20e934b27ba3ff927c1b4c9ff8cf6c32f315f743efe68dd3441182a87fb085` |
| `caddy` | `rawrarr-prod-caddy` | Running, healthy | `sha256:3067cc4e02c091399edbd5d65f85a3ff4d511574aecadb47f280c3761e68852e` |
| `cloudflared` | `rawrarr-prod-cloudflared` | Running | `sha256:1475624a6d235381d8886c0e0a7ca8694d01ae535370047a09c89787380e9d6c` |
| `swarmsail` | `rawrarr-prod-swarmsail` | Running | `sha256:107ead296b9b5463ab6468123b57f2ea9f54383272ae8d50c647b509a3e48713` |
| `crowdsec` | `rawrarr-prod-crowdsec` | Running, healthy | `sha256:5b37d0928f5b8600d2a09b1414471f77cbf2615be5e403311e66eb188852b018` |
| `umami` | `rawrarr-prod-umami` | Running | `sha256:341cf1f89cdf5326193015875c287bdb7a4d8b036156c3349268a86921f0206b` |
| `stripe-service` | `rawrarr-prod-stripe-service` | Running | `sha256:7f6539604372189cdfa85e6b546a57db53fbf285c0ba11b285353607ece41c49` |
| `headscale` | `rawrarr-prod-headscale` | Running | `sha256:46a8a7728a6fa2774e093313fa587922f68e1b56dc82f8276117050f27661939` |
| `tailscale` | `rawrarr-prod-tailscale` | Running | `sha256:9cb91ed119a2c502813cfef5a14430944163dfb2b290ccfc69d671d1469ec09d` |
| `roarlay` | `rawrarr-prod-roarlay` | Running, healthy | `sha256:e170627ecedd5b7cf321fc2318a134cf7373d167591227ab007160aab600bb66` |
| `subscriber-provisioner` | `rawrarr-prod-subscriber-provisioner` | Created; remains stopped | `sha256:c4947d56b51b3d36a11999526b01a12cb25641d19b95566204a39fe7ec9f77de` |

Before/after service records confirmed unchanged mounts, volumes, and network attachments for every rename. Preserved persistent volumes include the production PostgreSQL data, Umami PostgreSQL data, CrowdSec configuration/data, Caddy access logs, Tailscale state, and Roarlay UI. The saved source snapshot, live Compose files, pre-rename metadata, and per-service deployment logs are under `/opt/rawrarr-roarlay-prod/builds/production-container-rename-5624a8ef/rollback/`. The empty Compose-created `rawrarr-roarlay-prod_default` network was rechecked with zero endpoints and removed.

PostgreSQL readiness and `SELECT 1` passed; Umami PostgreSQL readiness passed; Caddy configuration validation passed; CrowdSec and Roarlay reported healthy. Production Tailscale reported `Running`, and Roarlay's network namespace matched the new Tailscale container. Production local `/health` and the internal coordinator HTTPS health route passed. The public `https://roarlay.rawrarr.net/health` edge continued returning the known baseline HTTP 403. No deployment locks remain.

After the renames, the authorized coordinator timer run exposed a missing Compose env-file argument. Issue [#407](https://github.com/willjasen/rawrarr/issues/407) fixed this in [PR #408](https://github.com/willjasen/rawrarr/pull/408). The updater was installed on do3 from published revision `c6698d0ee0e81c4d63893cf237e02a6d9624a992` (archive SHA-256 `7cd7c3c7b1570c22b9c55953675e93dbe8e5f218b3d47d9b1e1a9574933c0c59`). Its timer completed successfully at 2026-11-03 11:13:49 UTC and is enabled and active. Both internal and production coordinators now run that published revision from image `rawrarr:c6698d0ee0e81c4d63893cf237e02a6d9624a992-amd64` (`sha256:edf81411d1d4b4f6df838d5496af4c8e7e1d71b1aad1cd0fe0e7b5808f9fe32a`) and are healthy. This additional app rollout was explicitly approved; it preserved existing mounts and persistent data. The previous updater script and coordinator rollback images remain available.
