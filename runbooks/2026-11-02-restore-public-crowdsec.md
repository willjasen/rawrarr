# Restore CrowdSec for the public Rawrarr instance

Issue: [#306](https://github.com/willjasen/rawrarr/issues/306)

## Objective and scope

Restore CrowdSec detection and HTTP blocking on the public Rawrarr ingress on do3. The target is the public `rawrarr-roarlay-prod` Compose project only. The internal `rawrarr-internal-do3` coordinator, its database, its Tailscale sidecar, and internal Mac fossil nodes are out of scope.

## Inventory

- Read-only do3 inspection on 2026-11-02 found no CrowdSec container or CrowdSec Docker volume. Public Caddy is `rawrarr-roarlay-prod-caddy`, runs Caddy `v2.11.7`, and joins the existing `rawrarr-roarlay-prod-control` bridge network (`172.30.99.0/24`). The public Caddyfile routes Rawrarr to `tailscale:8080`, Headscale paths to `headscale:8080`, Stripe webhooks to `stripe-service:8090`, and the legacy `shale-mammoth-roarlay.willjasen.net` host to `headscale-swarmsail-1:8080`. Preserve these routes and the Caddy admin Unix socket.
- The retained historical CrowdSec files are on Brandon's MacBook Pro under `~/Library/Application Support/Swarmsail/crowdsec/`. The old database was last modified 2026-10-22 and the retained configuration includes local API and online API credentials. Its Docker daemon was unavailable during this inspection; the running CrowdSec version and data compatibility could not be verified.
- Keep that historical directory untouched. Start a fresh public do3 CrowdSec database rather than copying an unverified database and credentials from the former local Swarmsail project. This avoids carrying over stale decisions and credentials that may be tied to the retired local deployment. This is a new service, with no existing do3 CrowdSec state to migrate.

## Design

- Add CrowdSec `v1.8.2`, persist `/etc/crowdsec` and `/var/lib/crowdsec/data` in public-project-specific volumes, and install the `crowdsecurity/caddy` collection.
- Build a custom Caddy `v2.11.7` image with the HTTP bouncer module pinned at `v0.14.1`. The Caddy bouncer talks to CrowdSec over the existing public control network. CrowdSec's LAPI has no published host port.
- Use the checked-in Caddyfile to preserve current public routes and put the bouncer before each reverse proxy or response. The Caddy process reads the bouncer key from a root-owned Docker secret file; never commit that file or the key.
- Write JSON access logs to a bounded public-project volume shared read-only with CrowdSec. Remove query strings and authentication/cookie headers from logged records. Skip access logs for Headscale paths, `/internal/swarmsail/*`, Stripe webhooks, and the legacy Headscale host. Retain no more than seven daily rotations (and enforce a 20 MiB rotation size).
- CrowdSec reads only those Caddy logs with the `caddy` parser collection. No Docker socket, internal service logs, or node data are mounted.

## Adoption and deployment

This change adds service configuration and three new Docker volumes; it does not alter Rawrarr schemas, synchronized `/shared` data, or internal deployments. Existing public state is untouched. Deploy only after the change is published on GitHub `main`, using the exact merged commit archive. The archive must remain at its deployment path because Compose references the checked-in Caddyfile and acquisition file there.

1. Check `/opt/rawrarr-roarlay-prod/.container-rebuild-caddy.lock` and `.container-rebuild-crowdsec.lock`. The only old marker observed was the stale `.container-rebuild-roarlay.lock.stale-20261028T012641Z`; it is not a live lock. Claim both service locks atomically before deployment, keep them through health and route checks, and remove only the locks created by this task.
2. From a host with authenticated GitHub access, download the merged `main` source archive for the full published SHA, verify its digest, and transfer it to a new unused `/opt/rawrarr-roarlay-prod/builds/github-main-<sha>-crowdsec/` directory. Verify the digest again after transfer. Set `CROWDSEC_SOURCE_ROOT` to the extracted archive root, `RAWRARR_SOURCE_SHA` to the full SHA, and `CROWDSEC_BOUNCER_SECRET_FILE` to `/opt/rawrarr-roarlay-prod/secrets/caddy-crowdsec-bouncer`.
3. Create the secret parent directory if needed with mode `0700`, then create the secret file empty with mode `0400`. Use the exact public Compose file list reported by the existing Caddy container's `com.docker.compose.project.config_files` label, then add `deploy/do3/crowdsec/compose.yml`; keep project name `rawrarr-roarlay-prod` for all commands. Keep all Compose commands inside the dedicated task tmux session `rawrarr-crowdsec-20261102`; do not use the shared `ai` session.
4. Check host disk space and `docker info --format '{{.OSType}}/{{.Architecture}}'`. Pull `crowdsecurity/crowdsec:v1.8.2`, build the pinned Caddy image from the published archive, and verify its version, architecture, full-SHA image tag, and `org.opencontainers.image.revision` label. Tag the current public Caddy image ID under `rawrarr-roarlay-prod-caddy:rollback-<sha>` before changing the service. The source Caddyfile on the host remains untouched; the overlay adds a nested read-only file mount from the published archive.
5. Start only CrowdSec with `docker compose up -d --no-deps --pull never crowdsec`. Wait for its health check and verify the `caddy` acquisition is loaded. Generate a random hex key in a shell variable, register it with `cscli bouncers add caddy --key "$key"` without printing it, write the key into the root-owned secret file, and unset the variable.
6. Recreate only public `caddy` with the same Compose files, `--no-deps --pull never`. The overlay binds Caddy to do3's public IPv4 `104.248.184.91` only; binding wildcard port 443 conflicts with do3's tailnet-only Tailscale Serve listener. Pass the Caddy launch script as one Compose command argument so `/bin/sh -ec` executes the whole script. The added dependency requires CrowdSec to be healthy. Do not recreate PostgreSQL, Tailscale, Headscale, Stripe, Swarmsail, or internal services.
7. Verify CrowdSec health, acquisition metrics with lines read and parsed, valid bouncer registration and recent API pull, Caddy's compiled version and `crowdsec` module, and the public HTTPS health route. Test an explicit short-lived decision using a temporary client container on an unused public-control-network IP; confirm the same request is blocked, remove the decision, wait at least two 15-second bouncer poll intervals, and confirm the request is allowed again. Never use the operator's public address for this test.
8. Release both task-owned locks after verification. Keep the rollback image tag and published source archive until the public health route and bouncer test pass.

## Rollback and recovery

If Caddy fails validation or the public route fails, exclude the new overlay from the Compose command and create a temporary rollback Compose file containing `services: { caddy: { image: rawrarr-roarlay-prod-caddy:rollback-<sha>, pull_policy: never } }`; recreate only Caddy with `--no-deps --pull never`. This restores the original host-mounted Caddyfile and image. If the new CrowdSec service is unhealthy, first return Caddy to its saved image/config, then stop only the new CrowdSec container. Keep the new CrowdSec volumes, access-log volume, and secret file intact for diagnosis; do not delete Docker volumes. A Caddy image without the bouncer module cannot load the new Caddyfile, so restore both image and config together.

The new CrowdSec database is persistent and starts empty. The old MacBook Pro CrowdSec data remains in place as a separate historical archive and is not automatically promoted or copied. Restoring that data later requires identifying its CrowdSec version, validating its database/config compatibility, and rotating/replacing credentials before any import.

## Verification record

- Read-only public do3 inventory and current Caddy `caddy validate` succeeded.
- `docker compose -f deploy/do3/crowdsec/compose.yml config --quiet` succeeded locally with representative deployment variables; the rendered model contains only Caddy and CrowdSec plus the documented persistent volumes and external control network.
- No tests were run. The new image, merged public Compose configuration, live CrowdSec acquisition, Caddy bouncer, public HTTPS route, and controlled decision test remain to be verified during published-main deployment.

## References

- [Caddy CrowdSec bouncer](https://github.com/hslatman/caddy-crowdsec-bouncer)
- [CrowdSec Docker installation](https://docs.crowdsec.net/u/getting_started/installation/docker/)
- [Caddy access logging](https://caddyserver.com/docs/caddyfile/directives/log) and [`log_skip`](https://caddyserver.com/docs/caddyfile/directives/log_skip)
