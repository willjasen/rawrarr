# Roarlay, apex, and metadata deployments

Implemented roles are selected with `RAWARR_ROLE`. Unknown, missing, combined, `service`, and `shard` values fail startup. The three roles use the same development image with separate initialization and routes; roarlay never initializes subscriber app state. Public release images remain planned.

The roarlay site now has an authenticated `/setup/node` preview. First coordinator-account registration opens it after passkey creation; returning sign-ins open the existing workspace, which links back to setup. This deployment still supports only its established single owner, not public multi-account signup. The browser can create or import an age identity, create or validate an age-encrypted ML-DSA signing-root backup, and download a Compose file for **metadata** or **apex** only. Its Compose generator rejects coordinator/roarlay and includes no PostgreSQL or private keys. The browser neither uploads those private materials to roarlay nor persists them in browser storage. The public root is not yet pinned as an active trust anchor, and node approvals do not yet verify its signature. The Compose preview uses `rawrarr:local`; it requires an image on the destination host. A public image and ordinary one-command subscriber installation remain pending.

The generated Compose shape uses Docker named volumes for node-local `/data` and shared `/shared`, loopback `127.0.0.1:7297` for the local UI, and an optional read-only media bind for apex. These Docker features are portable across Docker Desktop on macOS/Windows and Docker Engine plus Compose on Debian, subject to host folder sharing and image architecture. It does not configure Syncthing or claim the node automatically: the owner must explicitly approve enrollment in the roarlay Devices view. Never use this preview to create a new roarlay.
User-facing names are roarlay (central role), apex (media storage role), and metadata (role name still under consideration). Compose services use `roarlay`, `apex`, and `metadata`; Docker generates names as `<project>-<service>-1`, except the local media override names its storage container `rawrarr-local-apex`. The `RAWARR_ROLE` values and database records retain `coordinator`, `storage`, and `metadata` for compatibility. The MacBook Tailscale sidecar retains the DNS alias `coordinator`.

| Capability | Roarlay (`coordinator`) | Apex (`storage`) | Metadata |
| --- | --- | --- | --- |
| Central passkeys, sessions, PostgreSQL | Yes | No | No |
| Approve/register nodes | Yes | No | No |
| Target primary workspace UI | Yes | No | No |
| Connect to Radarr | No | Yes | Yes |
| `/shared` and Syncthing participation | No | Yes | Yes |
| `/media` access | No | Read-only | No |
| Private age identity | Never | Local opt-in | Local opt-in |

“Subscriber” describes ownership, not a role. Metadata replaces shard. The former service role is superseded by the central coordinator; SwarmSail/Headscale and subscriptions remain future operator integrations. The coordinator does not hold media or private age identities. After roarlay-origin passkey sign-in, its workspace lists owner-authorized metadata and apex nodes, displays pending approvals, and shows a selected node's page through `/nodes/{node-id}/`. `localhost:7297` remains available for direct local access.

## Authentication and current trust scope

Roarlay is assumed to be already provisioned and healthy, and currently forms a **single-owner trust domain**. The existing MacBook metadata node is the seeded initial participant in roarlay's protected approved-node registry; it is not part of the joining flow. Later metadata or apex nodes create a random 256-bit credential locally and submit only its one-way SHA-256 verifier with role, ID, display name, and browser origin. The coordinator assigns each pending request to its sole account; the joining node cannot submit a user ID. The approval list and action are filtered to the signed-in owner, and each card names that account. Enrollment is rejected when roarlay has no account or multiple accounts. A future multi-account coordinator must use a short-lived invitation created by the intended owner. Roarlay never receives an age identity, media, shared data, or node-local app data.

Each node forwards only allowlisted authentication requests to roarlay. Its random credential is stored in node-local `/data`; roarlay stores the SHA-256 verifier plus approved role, node ID and exact WebAuthn RP/origin. PostgreSQL stores passkeys, expiring ceremonies/sessions, approved nodes, and short-lived pending requests. Sessions and challenges are scoped to the approved node ID. Subscriber age identities, Radarr credentials, metadata and media never enter the authentication proxy or enrollment request.

A joining node registers on startup. If it has no approved credential, it creates one locally, submits an enrollment request that expires after 15 minutes, then waits for explicit owner approval. The roarlay workspace displays pending requests and lets the signed-in owner approve them. The request does not include a user ID, age identity, or age-identity path. Incorrect/revoked credentials or role mismatches prevent startup. Authentication fails closed when roarlay is offline. Health is process liveness, not proof of authentication, database, sync, or integration availability. Existing authenticated long-lived event streams are not terminated immediately; stop the affected node for immediate revocation.

Standalone node browsing continues to use the node's origin. Existing `localhost:7297` passkeys remain compatible on that origin. The roarlay workspace uses a separate passkey explicitly linked to the existing account for `rawrarr.risk-mermaid.ts.net`; a localhost credential cannot sign in to that hostname. Within the signed-in workspace, roarlay grants a separate expiring session scoped to the selected node. Roarlay transport must use HTTPS outside the explicitly enabled local Docker development network. Redirects are not followed.

Roarlay relays only to configured targets from `RAWARR_NODE_RELAY_TARGETS`, a JSON object mapping registered node IDs to trusted HTTP(S) base URLs. The browser cannot supply an upstream URL. The default local Compose targets map `macbook-metadata` to `http://metadata:8080` and `mac-mini-storage` to `https://rawrarr-mini.risk-mermaid.ts.net`; the MacBook Tailscale overlay changes its local metadata target to `http://127.0.0.1:8081`. `RAWARR_NODE_DIRECT_TARGETS` advertises separate, narrow `/direct-api/` endpoints for browser use. The browser probes a direct target and uses the scoped relay if direct access fails. Both Tailscale containers provide local SOCKS5 proxies for cross-node traffic. The Mini Serve endpoint forwards to a loopback Caddy bridge, which reaches storage over the Mini's Docker network. Do not route `/media`, private age identities, `/shared`, or node-local data through roarlay. Library responses use per-request browser encryption, and export reads return age ciphertext; review the [blind coordinator content flow](blind-coordinator-content.md) before extending subscriber data routes.

## Local MacBook deployment

`docker-compose.yml` runs the already-established central `roarlay` + PostgreSQL and the seeded **metadata** participant. It preserves the existing `rawrarr-data`, `rawrarr-postgres-data`, shared bind mount and loopback port **7297**; the `metadata` service mounts the same existing `rawrarr-data` named volume. Roarlay's diagnostic/API port is loopback **7298**; it mounts only the PostgreSQL secret and approved-node registry, never `/data`, `/shared`, `/media` or the age identity. Syncthing remains its separate companion.

The MacBook tailnet routes have different backends by protocol. Roarlay shares the Tailscale container's network namespace, so HTTPS 443 `/` proxies directly to `http://127.0.0.1:8080`; no Caddy roarlay-proxy container is needed. A narrow `/direct-api/` Serve handler can reach the metadata listener at `http://127.0.0.1:8082`. Host diagnostics remain available only at `http://127.0.0.1:7298/`. Syncthing peer traffic uses a separate TCP 22000 forward directly to the Syncthing container when configured; its GUI is loopback-only at `http://127.0.0.1:8385/` and has no Serve web handler. Confirm the live Serve JSON before changing or claiming any handler.

Provision once (default is a dry run):

```sh
python3 scripts/provision_node.py --node-id macbook-metadata --role metadata
python3 scripts/provision_node.py --node-id macbook-metadata --role metadata --apply
RAWARR_POSTGRES_PASSWORD_FILE="$HOME/Library/Application Support/rawrarr/orbstack-postgres-password" \
  docker compose -p rawrarr-local -f docker-compose.yml -f docker-compose.tailscale-macbook.yml config --quiet
```

Before updating the persistent project, read each running service's `com.docker.compose.project.config_files` label and use its existing file set with `-p rawrarr-local`; the roarlay and metadata containers can record different sets. The MacBook overlay supplies the 7297 loopback listener and namespace sharing; base Compose alone does not. Follow the per-container deployment locks in [project instructions](../AGENTS.md), check port ownership, and preserve existing volumes. The additive migration scopes existing sessions/ceremonies to `macbook-metadata`; passkey records and encrypted shared data are unchanged. Do not rerun provisioning for an existing ID or replace its token. Never use `down -v`.

## Prepare the Mac mini

The Mini runs `docker-compose.storage.yml`, with no PostgreSQL or roarlay container. It does not create an account or require the node to know a username: the healthy coordinator maps it to its existing owner and exposes it for approval.

1. On the Mini, verify OrbStack, free loopback port 7297, and the real `/Volumes/silo/Media/data` mount. Preserve other consumers of the disk. Obtain the source checkout or a tested locally built image; public registry publishing remains planned.

2. Combine `docker-compose.storage.yml` with `docker-compose.tailscale-mini.yml`. The persistent Mini Tailscale container provides a userspace SOCKS5 proxy for roarlay bridge and Tailscale Serve for the storage page. Keep the Mini Serve endpoint private to the tailnet. PostgreSQL stays on roarlay's private Docker network and is never exposed through Tailscale or a public listener. The Mini browser origin remains `http://localhost:7297` for direct local access.

3. Start apex using only the standalone file and let it submit its enrollment request. Do not configure Syncthing peers, accept devices, or share a folder during initial deployment. The node must remain pending until the owner reviews and approves it in roarlay workspace.

   ```sh
   export RAWARR_COORDINATOR_URL=https://rawrarr.risk-mermaid.ts.net
   export RAWARR_COORDINATOR_LOCAL_HTTP=true
   export RAWARR_NODE_ID=mac-mini-storage
   export RAWARR_DEVICE_NAME="Mac mini"
   export RAWARR_NODE_ENROLLMENT_ENABLED=true
   export RAWARR_HOST_MEDIA_PATH=/Volumes/silo/Media/data
   docker compose -p rawrarr-local -f docker-compose.storage.yml config --quiet
   docker compose -p rawrarr-local -f docker-compose.storage.yml up -d --build
   ```

   The media bind is read-only and refuses to create a missing host path. The first startup creates a random node token in local `/data` and waits for approval. The coordinator associates the request with its existing owner; the node sends no username or user ID. Approve **Mac mini · apex node** in roarlay workspace. The approval card names the account it will join. Then registration completes. On the Mini, open `http://localhost:7297` and sign in using the localhost passkey. Unlock the age identity locally only if needed; it is neither requested by nor sent to roarlay. Do not mount the identity into roarlay or Syncthing.

4. Only after explicit owner approval, configure Syncthing on the approved node and its intended counterpart. Use distinct persistent container state, verify device IDs out of band, and share only the intended folder with the same folder ID. Set `RAWARR_SYNCTHING_HOSTNAME` and `RAWARR_SHARED_DATA_PATH` as needed; each GUI is loopback 8385. Never auto-accept a device or folder based on network presence or enrollment alone. Do not use or modify host Syncthing. Never sync `/data`, PostgreSQL, node tokens, or private age identities. Confirm the existing `config.aged` and public `age-recipient` arrive and synchronization is error-free. Do not create a new age identity for this existing instance. A separate, explicitly selected media directory may be replicated between approved storage nodes with the [media-sharing procedure](user-guide.md#replicate-a-storage-nodes-media-folder); it is never part of `rawrarr-shared`.

5. Verify `/health`, passkey sign-in, Radarr access, and shared-folder state. Enrolled storage nodes automatically scan every 24 hours by default, beginning one day after startup, and skip unchanged files that already have encrypted sidecars. Set `RAWARR_STORAGE_SCAN_INTERVAL_SECONDS` from 60 through 604800 to choose another interval, or set it to `0` to disable. The owner can request a full media scan or a bounded movie sample from the Devices view. Confirm encrypted sidecar propagation in both directions. A successful local scan/sync does not implement distributed catalog reconciliation or multi-library aggregation.

## Other shapes and recovery

- `docker-compose.coordinator.yml`: central roarlay + PostgreSQL only; same protected registry requirement.
- `docker-compose.metadata.yml`: standalone metadata node against an existing coordinator; configures its coordinator URL independently, creates a local token and enrolls for explicit approval; no media, PostgreSQL, or coordinator service dependency. As with storage, do not configure Syncthing peers or folders until the owner has approved that node.
- `docker-compose.media.yml`: local-development override changing `metadata` to storage with a read-only media bind. Provision a matching apex credential and set `RAWARR_NODE_TOKEN_PATH`; the metadata credential intentionally fails role validation.
- `storage-scan-once`: retained as a manually invoked local storage utility, with `RAWARR_ROLE=storage`; encrypts with the shared public recipient without web/auth/database startup. The enrolled storage service also runs this bounded scanner in the background. Errors and truncated scans are reported and retried after the interval; existing sidecars are not removed by an empty or failed scan.

Rollback the app image/Compose to the saved pre-migration version while preserving all volumes and the shared folder. The new database schema is additive; older SQL remains compatible, but the old application image's SQLx migration handling may reject an unknown newer migration. Prefer rolling forward; restoring the protected database backup requires stopping authentication writers and an explicit reviewed restore. Stop only new failed services; never delete data or relax mount permissions to resolve startup errors.

Roarlay serves a passkey sign-in page at `/` when signed out and the owner workspace after sign-in. The workspace offers full Movies and TV inventories, node status, selected-node settings, and all-indexed-movie metadata export. Library inventory is encrypted to a one-request browser session; export endpoints return age ciphertext for browser-side decryption. Only configured node targets and scoped API paths may be relayed. `/health` remains liveness and does not assert database or integration readiness. Cross-node catalog reconciliation and a production-scale streaming export remain planned. Node API authentication remains required.
