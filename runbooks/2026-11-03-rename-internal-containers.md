# Rename internal deployment containers

Issue: [#401](https://github.com/willjasen/rawrarr/issues/401)

## Scope and compatibility

Rename the internal deployment's containers to `rawrarr-int-*` while keeping the existing `rawrarr-internal-do3` Compose project, service names, named volumes, networks, secret paths, and Tailscale identity unchanged. The Caddy container is `rawrarr-int-caddy`: it serves the internal `/analytics` route and proxies other requests to the coordinator. The public deployment and Mac subscriber nodes are out of scope.

The target names are:

- `rawrarr-int-postgres`
- `rawrarr-int-tailscale`
- `rawrarr-int-roarlay`
- `rawrarr-int-caddy`
- `rawrarr-int-umami`
- `rawrarr-int-umami-postgres`

Container names are not persistent state. Compose will replace containers whose configured names change, while the existing named volumes retain PostgreSQL, Tailscale, UI, and analytics data. Do not remove volumes or start optional analytics services that were not already running. There is no data, schema, or protocol migration.

## Published-source rollout

1. Use only a verified GitHub `main` archive for the merged revision. Confirm do3 is `linux/amd64`, inspect free storage, and classify the published revision against the currently deployed coordinator revision. This change only updates deployment definitions and helper references, so reuse the running immutable app image; build a new app image only if the published app/runtime source requires one. Preserve the current app image under a rollback tag if an image build is needed.
2. Record the active Compose files, service/container labels, image IDs, mounts, volume names, health, and Tailscale Serve status. Preserve copies of the active internal Compose files and updater script outside the repository. Do not alter the public Compose project.
3. Stop the coordinator auto-update timer and wait for any active updater run to finish. Check all affected per-service deployment markers; wait rather than removing another task's marker. Keep deployment locks held through recreation and verification, acquiring each service's marker atomically immediately before changing that service.
4. Install the new internal Compose definitions and the `auto_update_coordinators.py` from the verified published archive. Before recreating anything, validate the merged Compose configuration against the existing protected environment and active override files. Keep the Compose project and volume/network definitions unchanged.
5. Recreate only services that were already active, in dependency order: PostgreSQL and optional Umami PostgreSQL/Umami first; Tailscale next; then roarlay and optional Caddy. Use the existing Compose project, `--no-deps`, and `--pull never`; do not recreate public services, subscriber nodes, or unrelated containers. For each service, hold its existing `.container-rebuild-<service>.lock` marker until the new container is verified.
6. Verify each expected container name and its Compose project/service labels. Confirm database, Tailscale, UI, and analytics mounts still point to the original volumes. Verify PostgreSQL health, coordinator app health and image revision/architecture, Tailscale Serve's `rawrarr.risk-mermaid.ts.net:443` `/` proxy to `http://127.0.0.1:8080`, and HTTPS `/health`. When analytics was active, verify Umami and its database health, plus the `/analytics` route through Caddy.
7. Release each task-owned lock after that service's verification. Re-enable the auto-update timer and verify it is active only after the new updater and Compose configuration are installed and the coordinator route is healthy.

If any verification fails, stop further changes. Restore the saved internal Compose files and updater script, then recreate the original service names under their deployment locks using the same Compose project and existing volumes. Verify the original coordinator health and Serve route before restarting the timer. Keep both versions of the app image and all volumes until rollback is complete; do not prune or delete runtime data.

## Execution record

Deployed from published `main` at `ecafe9d1542c656bb6b77c19ca379e062e3fd10e`. The authenticated source archive SHA-256 was `433dee92cade5f359163c5b6e7287a69617c0e281d92c4905491917d4377c63e`. The change did not affect app/runtime code, so no image was built: `rawrarr:e1ff1038a9c656a8d2e2517e650086abcdbd4f38-amd64` remained in use with image ID `sha256:e170627ecedd5b7cf321fc2318a134cf7373d167591227ab007160aab600bb66`, revision `e1ff1038a9c656a8d2e2517e650086abcdbd4f38`, architecture `linux/amd64`.

Recreated the six already-active internal services only: `postgres`, `umami-postgres`, `umami`, `tailscale`, `roarlay`, and `analytics-caddy`. Their resulting container names are the six `rawrarr-int-*` names listed above. The existing volumes remained mounted: `rawrarr-internal-do3-postgres-data`, `rawrarr-internal-do3-tailscale-state`, `rawrarr-internal-do3-ui`, and `rawrarr-internal-do3_umami-postgres-data`. The `rawrarr-internal-do3` Compose project, service names, secrets, and Tailscale identity were unchanged. No public or subscriber services were touched.

Verification passed: PostgreSQL and Umami PostgreSQL reported healthy; roarlay reported healthy; Umami, Caddy, and Tailscale were running. Tailscale Serve retained `/` → `http://127.0.0.1:8080` and `/analytics` → `http://127.0.0.1:8081/analytics`. The internal HTTPS `/health` endpoint returned HTTP 200 and identified the responding Roarlay web service; `/analytics` returned HTTP 200. The auto-update timer was restarted and is active. Its first run succeeded and reported that `ecafe9d1542c656bb6b77c19ca379e062e3fd10e` contains no coordinator runtime changes since the deployed app revision. All six deployment locks were released after verification.

No rollback was required. The pre-deployment Compose and updater backups remain under `/opt/rawrarr-internal-do3/builds/container-rename-ecafe9d/rollback/`; all original volumes and the deployed app image remain intact.
