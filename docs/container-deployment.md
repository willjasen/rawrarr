# Roarlay, fossil, and boneyard deployments

See the [Compose inventory](compose-layout.md) before choosing or changing a
definition. The root files include historical Mac topologies, not just current
deployments. The isolated two-fossil test stack is now
[`deploy/testing/compose.yml`](../deploy/testing/compose.yml); do3 definitions
remain under `deploy/do3/`. Existing operational commands must keep their recorded
Compose project, full overlay combination, and persistent resources.

## Fast internal-to-public coordinator promotion on do3

The do3 updater pulls the exact published `linux/amd64` image for each source revision from the public GHCR package, then promotes that same verified image ID from internal to public/live. The internal coordinator uses the `rawrarr-int-*` container names in the `rawrarr-internal-do3` Compose project; the public/live coordinator uses `rawrarr-prod-*` names in the `rawrarr-roarlay-prod` project, supplied by `docker-compose.production-container-names.yml`. Include this overlay in production Compose commands so future recreations preserve the names. Their Compose projects, databases, configuration, secrets, and deployment locks stay separate.

The current internal container names are `rawrarr-int-postgres`, `rawrarr-int-tailscale`, `rawrarr-int-roarlay`, `rawrarr-int-caddy`, `rawrarr-int-umami`, and `rawrarr-int-umami-postgres`. The production names are `rawrarr-prod-postgres`, `rawrarr-prod-umami-postgres`, `rawrarr-prod-caddy`, `rawrarr-prod-cloudflared`, `rawrarr-prod-swarmsail`, `rawrarr-prod-crowdsec`, `rawrarr-prod-umami`, `rawrarr-prod-stripe-service`, `rawrarr-prod-headscale`, `rawrarr-prod-tailscale`, `rawrarr-prod-roarlay`, and `rawrarr-prod-subscriber-provisioner`. Production names come from [`docker-compose.production-container-names.yml`](../deploy/do3/docker-compose.production-container-names.yml); include that overlay in production Compose commands that recreate services. These are container names only: Compose project names, service names, volumes, networks, and configuration remain unchanged. The subscriber-provisioner remains stopped unless explicitly started for provisioning.

On do3, `rawrarr-coordinator-auto-update.timer` checks GitHub `main` once a minute and deploys published coordinator/runtime changes. It reads the host-local `/root/gh-token` file for GitHub revision and compare metadata, coalesces rapid commits to the newest published app image, and skips documentation, test, workflow, landing-page, and deployment-only Compose definitions, including the internal coordinator/dashboard and production container-name overlays. Those Compose definitions are applied by their deployment runbooks, not by changing the app image. It consumes only the published `main` ref; do not use this service to deploy a branch or dirty working tree. The host token has Contents read access and cannot inspect Actions check runs, so the repository's normal review and merge checks remain the gate for changes entering `main`. Image discovery pulls the public GHCR `main` tag without GitHub credentials, verifies its `linux/amd64` platform and source revision label, and compares that revision to current GitHub `main`. If coordinator/runtime changes exist after the published image revision, deployment waits for the image workflow and retries on a later poll. Otherwise it pins the image locally by its immutable `sha-<image-revision>` tag, recreates internal `roarlay`, verifies it, then promotes the same image ID to public/live. The service records a per-environment `RAWARR_DEPLOYED_AT` value for the coordinator footer. See [`auto_update_coordinators.py`](../deploy/do3/auto_update_coordinators.py) and the [installation runbook](../runbooks/2026-11-03-deploy-coordinator-auto-updater.md).

The container publish workflow creates an architecture-specific image and, after both platform builds finish, publishes a multi-architecture `sha-<full-sha>` manifest and advances the mutable `main` tag. The updater can observe source changes before that workflow finishes; if the currently published image is behind `main` for coordinator/runtime paths, it waits rather than deploying stale code. Deploying a published image avoids compiling Rawrarr on do3 and does not require build-cache space or registry authentication.

For each promotion:

1. Publish and merge the change to GitHub `main`. Do not build from a feature branch or a working tree. Pull `ghcr.io/willjasen/rawrarr:main` with `--platform linux/amd64`, verify the image ID, `org.opencontainers.image.revision` label, and architecture, and compare the labeled source revision to `main`. Proceed only if no coordinator/runtime paths changed after the image revision, then use the corresponding immutable `sha-<image-revision>` tag. Do not compile the Rawrarr application on do3.
2. Preserve the prior service image ID and `rawrarr:internal-base-amd64` image under rollback tags, then advance the reusable base tag only after the published image passes revision and platform checks. The pulled immutable SHA image is the promotion artifact.
3. Atomically claim the internal roarlay deployment lock at `/opt/rawrarr-internal-do3/.container-rebuild-roarlay.lock`. Use the existing internal Compose project and host-local image override with `pull_policy: never`; recreate only `roarlay` with `--no-deps --pull never`. Verify its container health, image revision/architecture, local health on port 7299, Tailscale Serve JSON mapping `rawrarr.risk-mermaid.ts.net:443` `/` to `http://127.0.0.1:8080`, and HTTPS `/health`. Release the lock after verification.
4. After internal verification, atomically claim `/opt/rawrarr-roarlay-prod/.container-rebuild-roarlay.lock`. Use the existing public Compose files and a host-local override pointing to the *same immutable SHA image*. Recreate only public `roarlay` with `--no-deps --pull never`. Verify health and the image revision/architecture, then verify `https://roarlay.rawrarr.net/health`. Release the lock after verification. Do not recreate PostgreSQL, Tailscale, Caddy, Cloudflare Tunnel, or unrelated services.

Use `scripts/classify_internal_deploy_targets.py --base <previously-deployed-sha> --head <published-sha>` to decide whether the commit also needs subscriber updates. Coordinator-only changes require no Mac build. If node code changed, follow the native per-architecture subscriber workflow separately; an amd64 do3 image cannot run on the arm64 Mac mini. UI and application code ship together in the same image. Changes anywhere under `rust-arr/` trigger the multi-architecture image workflow; the coordinator auto-updater consumes published `main`, while subscriber deployment follows the architecture-native node workflow. There is no separate UI-only release or runtime polling path.

If internal verification fails, leave public/live on its prior image and roll the internal service back to its saved image tag under the same lock. If public verification fails after promotion, roll back only public roarlay. Keep both environments' databases and volumes in place. For database/schema changes, follow the feature's compatibility and backup plan; an older binary may not start after a newer SQLx migration.

The workflow changes no existing persistent data or protocol by itself and requires no migration. It adopts code through the same published image path already used by each coordinator.

## Refresh do3 support images

Use [`update_support_images.py`](../deploy/do3/update_support_images.py) to refresh the third-party support images declared in the internal and public support-image Compose overlays. The script recovers each running service's exact Compose project and active files from Docker labels, validates the merged image against the checked-in overlay, pulls only that service, and recreates only that service under its per-service lock. It passes each project's protected `secrets/analytics.env` file to Compose when present, uses the running internal roarlay image for internal interpolation, and derives public CrowdSec source and secret-file paths from the active Compose files; it never reads or logs secret values. It does not change or recreate roarlay. Its overlay temporarily permits pulling the selected image, including images whose normal deployment config says `pull_policy: never`; recreation still uses `--pull never`. It preserves the previous image under a rollback tag, skips recreation when the image ID is unchanged, and checks container readiness plus the affected environment's health route. Do not run generic `docker pull`/`docker compose up` commands for this task.

The overlays intentionally preserve configured versions and digests for PostgreSQL 17, Headscale, CrowdSec, Umami, and cloudflared. Mutable `caddy:2-alpine` and `tailscale/tailscale:stable` tags move to their registry-current image when refreshed; PostgreSQL remains within major version 17. Updating a pinned upstream version or digest requires a separate reviewed repository change. Rawrarr-built services and local custom images are excluded. Both environments' local roarlay health endpoints are checked; the internal Serve route must also pass. Cloudflare may return HTTP 403 for the public edge health URL when called from do3, so that edge response is logged as a warning rather than rolling back a healthy local service.

On do3, run the updater only from an authenticated, digest-verified archive of the published `main` commit. Check disk space and locks first; do not prune images, volumes, or BuildKit cache. The updater updates services sequentially and never restarts dependencies or unrelated projects. PostgreSQL data volumes and Tailscale state volumes remain attached. See the [support image refresh runbook](../runbooks/2026-11-03-refresh-do3-support-images.md).

Implemented roles are selected with `RAWARR_ROLE`. Unknown, missing, combined, `service`, and `shard` values fail startup. The three roles use the same image with separate initialization and routes; roarlay never initializes subscriber app state. GitHub Actions publishes `ghcr.io/willjasen/rawrarr` images for Linux amd64 and arm64 on relevant pushes to `main`; the package is public so subscriber nodes can pull without a GitHub account or token. `main` tracks the latest successful build and the commit SHA tag is immutable.

The roarlay site now has an authenticated `/setup/node` preview. First coordinator-account registration opens it after passkey creation; returning sign-ins open the existing workspace, which links back to setup. This deployment still supports only its established single owner, not public multi-account signup. The browser can create or import an age identity, create or validate an age-encrypted ML-DSA signing-root backup, and download a Compose file for **fossil** (metadata) or **boneyard** (storage) only. Its Compose generator rejects coordinator/roarlay and includes no PostgreSQL or private keys. The browser neither uploads those private materials to roarlay nor persists them in browser storage. The public root is not yet pinned as an active trust anchor, and node approvals do not yet verify its signature. Generated Compose uses the public `ghcr.io/willjasen/rawrarr:main` image with `pull_policy: always`, so each `docker compose up -d` checks GHCR for the latest main image. Subscriber nodes do not need a GitHub account, token, or `docker login`; coordinator-side GitHub credentials, if configured for coordinator features, are separate and unchanged.

The generated Compose shape uses Docker named volumes for node-local `/data` and shared `/shared`, loopback `127.0.0.1:7297` for the local UI, and an optional read-only media bind for boneyard. These Docker features are portable across Docker Desktop on macOS/Windows and Docker Engine plus Compose on Debian, subject to host folder sharing and image architecture. Local Docker Compose builds use BuildKit's target architecture to compile the Rust binary (`amd64` or `arm64`) and reject unsupported targets; on a normal local Docker context, that target follows the Docker host. It does not configure Syncthing or claim the node automatically: the owner must explicitly approve enrollment in the roarlay Devices view. Never use this preview to create a new roarlay.

UI templates, browser modules, and assets are embedded in the Rawrarr application image. GitHub Actions builds and publishes both Linux architectures when `rust-arr/**` changes, so UI changes travel with the corresponding application revision. The Rust service serves the assets compiled into its image; legacy `/ui` volumes may remain mounted during rollout but are ignored and cannot shadow the image. No GitHub release token or UI polling process is needed. Existing `/ui` volume data is preserved and can be removed separately after deployment owners confirm it is no longer needed.

The [one-line node deployment plan](one-line-node-deployment-plan.md) proposes a copied curl command that downloads a script containing this generated Compose definition, checks or helps install Docker and Compose, and starts the subscriber node. It remains planned, depends on published images, and does not replace owner approval or the manual Compose path.

User-facing names are roarlay (coordinator), fossil (metadata), and boneyard (storage). Compose services use `roarlay`, `fossil`, and `boneyard`; Docker generates names as `<project>-<service>-1`. The `RAWARR_ROLE` values and database records retain `coordinator`, `metadata`, and `storage` for compatibility. The internal coordinator stays on do3; neither Mac is a coordinator deployment target.

| Capability | Roarlay (`coordinator`) | Fossil (`metadata`) | Boneyard (`storage`) |
| --- | --- | --- | --- |
| Central passkeys, sessions, PostgreSQL | Yes | No | No |
| Approve/register nodes | Yes | No | No |
| Target primary workspace UI | Yes | No | No |
| Connect to Radarr | No | Yes | Yes |
| `/shared` and Syncthing participation | No | Yes | Yes |
| `/media` access | No | No | Read-only |
| Private age identity | Never | Local opt-in | Local opt-in |

“Subscriber” describes ownership, not a role. Metadata replaces shard. The former service role is superseded by the central coordinator; subscription eligibility and automatic Headscale provisioning remain future integrations. The coordinator does not hold media or private age identities. After roarlay-origin passkey sign-in, its workspace lists owner-authorized fossil and boneyard nodes, displays pending approvals, and shows a selected node's page through `/nodes/{node-id}/`. `localhost:7297` remains available for direct local access.

## Retired Swarmsail Headscale staging

The October 18 MacBook/Mini Headscale staging definitions were retired on
2026-11-03 with explicit operator confirmation that both are obsolete.
`docker-compose.headscale-macbook.yml` and `docker-compose.headscale-mini.yml`
are no longer installation inputs. The old LAN coordinator/control-plane routes
in the [staging runbook](../runbooks/2026-10-18-headscale-container-client-staging.md)
describe that historical experiment, not the current internal environment.

This removes repository templates only; it does not stop sidecars, revoke
enrollment, or delete node-local identity state. App Tailscale companions,
do3 public Headscale services, and the isolated test Headscale fixture are
unchanged. See [retirement and recovery guidance](compose-layout.md#retired-staged-headscale-clients).

## Authentication and current trust scope

Roarlay is assumed to be already provisioned and healthy. Its public auth page permits multiple independent passkey accounts, but node enrollment still forms a **single-owner trust domain**. The existing MacBook fossil node is the seeded initial participant in roarlay's protected approved-node registry; it is not part of the joining flow. Later fossil or boneyard nodes create a random 256-bit credential locally and submit only its one-way SHA-256 verifier with role, ID, display name, and browser origin. The coordinator assigns each pending request to its sole account; the joining node cannot submit a user ID. The approval list and action are filtered to the signed-in owner, and each card names that account. Enrollment is rejected when roarlay has no account or multiple accounts. A future multi-account coordinator must use a short-lived invitation created by the intended owner. Roarlay never receives an age identity, media, shared data, or node-local app data.

Each node forwards only allowlisted authentication requests to roarlay. Its random credential is stored in node-local `/data`; roarlay stores the SHA-256 verifier plus approved role, node ID and exact WebAuthn RP/origin. PostgreSQL stores passkeys, expiring ceremonies/sessions, approved nodes, and short-lived pending requests. Sessions and challenges are scoped to the approved node ID. Subscriber age identities, Radarr credentials, metadata and media never enter the authentication proxy or enrollment request.

A joining node registers on startup. If it has no approved credential, it creates one locally, submits an enrollment request that expires after 15 minutes, then waits for explicit owner approval. The roarlay workspace displays pending requests and lets the signed-in owner approve them. The request does not include a user ID, age identity, or age-identity path. Incorrect/revoked credentials or role mismatches prevent startup. Authentication fails closed when roarlay is offline. Health is process liveness, not proof of authentication, database, sync, or integration availability. Existing authenticated long-lived event streams are not terminated immediately; stop the affected node for immediate revocation.

Standalone node browsing continues to use the node's origin. Existing `localhost:7297` passkeys remain compatible on that origin. The roarlay workspace uses a separate passkey explicitly linked to the existing account for `rawrarr.risk-mermaid.ts.net`; a localhost credential cannot sign in to that hostname. Within the signed-in workspace, roarlay grants a separate expiring session scoped to the selected node. Roarlay transport must use HTTPS outside the explicitly enabled local Docker development network. Redirects are not followed.

Roarlay relays only to configured targets from `RAWARR_NODE_RELAY_TARGETS`, a JSON object mapping registered node IDs to trusted HTTP(S) base URLs. The browser cannot supply an upstream URL. The do3 internal definition maps the Mac subscribers to their own HTTPS tailnet hostnames. `RAWARR_NODE_DIRECT_TARGETS` advertises separate, narrow `/direct-api/` endpoints for browser use. The browser probes a direct target and uses the scoped relay if direct access fails. Subscriber transport must reach the coordinator over its configured HTTPS endpoint; no Mac-local coordinator is needed. Do not route `/media`, private age identities, `/shared`, or node-local data through roarlay. Library responses use per-request browser encryption, and export reads return age ciphertext; review the [blind coordinator content flow](blind-coordinator-content.md) before extending subscriber data routes.

## Mac subscribers, not coordinators

The MacBook and Mac mini are subscriber hosts. Their internal fossil deployments
use each host's established `rawrarr-node-internal` project and host-local Compose
files; inspect container labels before maintenance and preserve volumes,
credentials, identities, and `/shared`. Coordinator deployment on either Mac is
retired. The internal coordinator remains on do3 at
`https://rawrarr.risk-mermaid.ts.net/`.

The generic root `docker-compose.yml` and `docker-compose.coordinator.yml` remain
self-hosting/development templates, not instructions to recreate the internal
environment on a Mac. Their defaults and volumes are not a migration mechanism.
The former `docker-compose.tailscale-macbook.yml` is removed, and the standalone
admin preview is paused with its Dockerfile and Compose launcher removed. See the
[admin preview adoption notes](admin-console.md#paused-preview) and the
[retirement record](compose-layout.md#retired-mac-coordinator-definitions).

## Retired Mac-specific storage setups

The internal coordinator moved from the Mac mini to do3 on 2026-10-27.
Its definition is
[`deploy/do3/docker-compose.internal-coordinator.yml`](../deploy/do3/docker-compose.internal-coordinator.yml);
the former `docker-compose.internal-mini.yml` has been retired. Do not start a
second coordinator on a Mac from historical runbook commands. The internal
coordinator remains at `https://rawrarr.risk-mermaid.ts.net/`; the Macs remain
subscriber hosts. See the [migration record](../runbooks/2026-10-27-move-internal-coordinator-to-do3.md)
and [retirement/adoption guidance](compose-layout.md#retired-mac-coordinator-definitions).
The old host-specific storage setup is not the current internal Mac deployment.

The operator confirmed the old MacBook boneyard and Mini transport/relay setups
obsolete. Their host-specific templates and Caddy fixture have been removed.
Do not replay their historical deployment commands or create replacement
Tailscale identities. The current MacBook `rawrarr-node-internal` stack contains
fossil, artwork-cache, and tailscale, using host-local files.

The generic `docker-compose.storage.yml` remains available for an intentionally
provisioned boneyard, not as a current Mac setup recipe. It requires an existing,
separately managed Tailscale namespace selected by `RAWARR_TAILSCALE_CONTAINER`;
its legacy fallback name does not provision that container. Inspect the actual
namespace and configure verified HTTPS/coordinator and browser routes before
deployment. The app binds loopback and publishes no host port, so starting it
alone does not make `localhost:7297` reachable.

Preserve the existing project's node-local `/data`, shared folder, media mounts,
credentials, and transport identity. Owner approval is required before sharing
Syncthing folders. No automatic migration or live teardown accompanies this
retirement. Unexpected old callers must stop and inspect their labels and state;
see [recovery guidance](compose-layout.md#retired-host-specific-storage-and-relays).

## Other shapes and recovery

- `docker-compose.coordinator.yml`: central roarlay + PostgreSQL only; same protected registry requirement.
- `docker-compose.metadata.yml`: standalone fossil (metadata) node against an existing coordinator; configures its coordinator URL independently, creates a local token and enrolls for explicit approval; no media, PostgreSQL, or coordinator service dependency. As with boneyard, do not configure Syncthing peers or folders until the owner has approved that node.
- `docker-compose.media.yml`: local-development override changing the `fossil` service to the storage role with a read-only media bind. Provision a matching boneyard credential and set `RAWARR_NODE_TOKEN_PATH`; the metadata credential intentionally fails role validation.
- `storage-scan-once`: retained as a manually invoked local storage utility, with `RAWARR_ROLE=storage`; encrypts with the shared public recipient without web/auth/database startup. The enrolled storage service also runs this bounded scanner in the background. Errors and truncated scans are reported and retried after the interval; existing sidecars are not removed by an empty or failed scan.

Rollback the app image/Compose to the saved pre-migration version while preserving all volumes and the shared folder. The new database schema is additive; older SQL remains compatible, but the old application image's SQLx migration handling may reject an unknown newer migration. Prefer rolling forward; restoring the protected database backup requires stopping authentication writers and an explicit reviewed restore. Stop only new failed services; never delete data or relax mount permissions to resolve startup errors.

Roarlay serves a passkey sign-in page at `/` when signed out and the owner workspace after sign-in. The workspace offers full Movies and TV inventories, node status, selected-node settings, and all-indexed-movie metadata export. Library inventory is encrypted to a one-request browser session; export endpoints return age ciphertext for browser-side decryption. Only configured node targets and scoped API paths may be relayed. `/health` remains liveness and does not assert database or integration readiness. Cross-node catalog reconciliation and a production-scale streaming export remain planned. Node API authentication remains required.
