# Rawrarr user guide

> Current container architecture: **roarlay** (`RAWARR_ROLE=coordinator`) owns passkeys, authentication, and node coordination; **fossil** (`metadata`) serves encrypted app data and Arr connections; **boneyard** (`storage`) reads local media. See [container deployment](container-deployment.md) for the role-specific Compose files and node enrollment.

This guide covers features available in the current Docker prototype. For completed and outstanding project work, see the [roadmap](../ROADMAP.md); for data and security constraints, see [technical status](technical-status.md).

## Start Rawrarr with Docker Compose

Requirements: Docker Compose, a local PostgreSQL password file, and a browser with passkey support. OrbStack provides Docker on macOS.

For the existing persistent deployment, reuse Compose project `rawrarr-local` and its protected password file. Do not generate a replacement password for its existing database. Before starting or rebuilding, inspect ownership of port 7297:

```sh
lsof -nP -iTCP:7297 -sTCP:LISTEN
docker ps --filter publish=7297 --format 'table {{.Names}}\t{{.Ports}}'
```

If the port belongs to the existing `rawrarr-local` app, update that project. Resolve any other owner before proceeding; do not choose another port or remove volumes to work around a conflict. From the repository root, identify the Compose file set recorded by each running service before an update:

```sh
export RAWARR_POSTGRES_PASSWORD_FILE="$HOME/Library/Application Support/rawrarr/orbstack-postgres-password"
test -f "$RAWARR_POSTGRES_PASSWORD_FILE"
docker inspect rawrarr-local-roarlay-1 --format '{{ index .Config.Labels "com.docker.compose.project.config_files" }}'
docker inspect rawrarr-local-fossil-1 --format '{{ index .Config.Labels "com.docker.compose.project.config_files" }}'
```

Use the recorded files and project name when updating a service; the MacBook Tailscale overlay supplies the 7297 listener and shared network namespace. The base `docker-compose.yml` alone does not publish port 7297. Follow the [deployment guide](container-deployment.md) and the per-container rebuild locks in [project instructions](../AGENTS.md) before recreating a service. If the password file is missing, inspect the existing Compose secret mount to locate it without reading its contents. For a genuinely new installation, provision a protected random PostgreSQL password file outside the repository first.

Open `https://rawrarr.risk-mermaid.ts.net/` for the roarlay workspace and its tailnet-origin passkey. In the current MacBook overlay, `127.0.0.1:7297` and `127.0.0.1:7298` both forward to roarlay for local diagnostics; the fossil service listens inside the shared Tailscale namespace on `127.0.0.1:8081`. The tailnet and `localhost` WebAuthn origins use separate credentials linked to the same account. Keep the origin and relying-party hostname aligned when configuring another hostname.

With the password-file variable set, useful commands are:

```sh
docker compose -p rawrarr-local logs -f fossil
docker compose -p rawrarr-local ps
curl --fail http://127.0.0.1:7297/health
```

The `/health` route returns an HTML web-process status page with a 30-second refresh. On 7297 in the current MacBook overlay it identifies roarlay. HTTP 200 confirms that the web process responds; it does not verify PostgreSQL queries, Radarr, Syncthing, other nodes, or end-to-end readiness. Check the Compose container health and the relevant integration separately.

The container logs request method, path, status, and duration by default. Request paths are logged without query parameters, and headers and bodies are never logged. Set `RAWARR_QUIET_LOGS=true` in the Compose environment to suppress routine request lines; server errors remain visible. Successful library scans also log numeric summary counts.

Compose stores the shared Rawrarr data in `~/Library/Application Support/rawrarr/shared-data` by default, outside the source repository. Open it in Finder with:

```sh
open "$HOME/Library/Application Support/rawrarr/shared-data"
```

To place it elsewhere, set `RAWARR_SHARED_DATA_PATH` to a persistent host directory before starting Compose. The PostgreSQL and `/data` state remain in Docker-managed volumes.

`docker compose -p rawrarr-local down` stops the services but preserves named volumes and the host shared-data folder. Do not use `docker compose -p rawrarr-local down -v` unless you intend to delete the local PostgreSQL and Rawrarr data volumes.

## First login and age identity

Roarlay owns account passkeys in its PostgreSQL database. Sign in through the tailnet URL; an existing `localhost` credential cannot be used at the tailnet origin without linking a passkey for that origin. In the roarlay workspace, enter the age private ID on Movies to unlock selected-node metadata and settings, or save it in browser storage from Settings by explicit choice. The ID otherwise stays in the tab's memory, and sign-out clears both locations.

The following generated/imported ID flow describes the fossil-node onboarding interface when that interface is exposed directly:

1. If PostgreSQL has no passkey account yet, enter your name and select **Create passkey**. Rawrarr creates the internal username from your name and stores the account and passkey in roarlay's PostgreSQL database. On later visits, the page shows only **Sign in with passkey** and uses the account already stored in PostgreSQL.
2. After passkey sign-in, Rawrarr opens age identity setup if the vault is locked. A remembered identity can unlock it during startup; passkey sign-in is still required. Select **Create ID** to generate an identity. The page shows a shortened preview and a public-recipient fingerprint, then opens a copy pop-up.
3. In the pop-up, select **Copy private identity** and save the key somewhere recoverable, such as a password manager. The clipboard receives only the private key line; the public recipient is not included. The full private identity is not rendered on the page or written to server logs.
4. After the copy succeeds, select **I copied it and saved the private ID somewhere safe.** Select **Continue** to unlock the encrypted settings and open the dashboard. The three-character check is temporarily disabled.
5. To use an existing post-quantum identity, choose **Import ID**, paste its private key into the pop-up, then choose **Unlock vault** directly, without the generated-key copy acknowledgement. Optionally select **Keep this ID on this device** before importing to avoid pasting it again after an app restart. The same option is available before unlocking a generated ID. The ID is saved with owner-only permissions at `/data/age-identity` in local Compose, never in `/shared`; passkey sign-in is still required. Rawrarr cannot recover a lost ID. An optional Docker secret mount is also described in `docker-compose.age-identity.yml`.

The public recipient is safe to share for encryption, but it cannot unlock the config. Never send the private identity to another node or service.

## Configure Radarr

1. Open the Rawrarr dashboard and its **Settings** section.
2. Enter the HTTPS Radarr base URL and API key, then choose **Save and connect**. Rawrarr rejects HTTP URLs and checks the Radarr status endpoint using those credentials before saving.
3. After a successful check, Rawrarr saves the settings into `/shared/config.aged` and applies the connection immediately; a Rawrarr restart is not required. In the local Compose environment, it also keeps a mode-`0600` plaintext copy at `/data/radarr-config.json` so local tools can inspect the setup without passkey or age handling. Changes saved through Rawrarr update both copies. When the age vault is unlocked, the shared encrypted file is authoritative and refreshes the local copy.
4. Use the connection status or **Movies** view to read Radarr's current movie inventory. The artwork source column reports shared-cache availability or Radarr fallback; it is not proof that a cached file decrypted successfully. The load-time column measures browser-observed poster load time, not isolated Radarr response time. Failed image requests show as unavailable.

Movie text records are stored at `/shared/metadata/movies/text/` and encrypted artwork at `/shared/metadata/movies/artwork/`. Posters use shared encrypted artwork when available and the vault is unlocked, with live Radarr fallback. Missing movie details and posters are cached on demand rather than scraping the entire library at once.

For Arr services using a private certificate authority, set `RAWARR_ARR_CA_CERT_FILE` on each subscriber to the PEM-encoded CA certificate. Rawrarr adds that CA to its existing trusted roots for Radarr and Sonarr HTTPS requests; it does not allow HTTP or disable certificate verification. Mount only the CA certificate into the subscriber. The CA private key stays with the service that issues certificates and is never needed by Rawrarr.

## Recover Radarr on a chosen node

When a Radarr connection is configured and the age vault is unlocked, Rawrarr asks Radarr to create a native backup on startup and every six hours. It age-encrypts the latest backup to `/shared/metadata/radarr/recovery.zip.aged`; Syncthing replicates that ciphertext to the other approved nodes. The archive contains Radarr's database and settings, including any credentials stored by Radarr. Rawrarr's scraped movie metadata and artwork are already stored separately in the shared, age-encrypted `movies/text/` and `movies/artwork/` folders, and remain available independently of the Radarr restore. If capture fails, the previous snapshot is retained and the failure is written to the node log.

If Radarr requires a UI login to download its backup ZIP, enter both the optional UI username and password in the Radarr connection settings. They are sent directly to the selected metadata node and stored only inside the age-encrypted shared config; they are never shown again. Leave both blank to keep a saved login, or use **Clear connection** to remove it. These credentials are used only for the configured Radarr HTTPS origin when downloading backups.

To restore, deploy a Radarr container yourself on the metadata or storage node you choose, wait for its shared folder to sync, then run the helper from a Rawrarr source checkout on a host with Python 3 and `age` installed:

```sh
python3 scripts/restore_radarr.py \
  --backup /path/to/shared/metadata/radarr/recovery.zip.aged \
  --identity-file /path/to/private/age-identity \
  --url https://radarr.example.com
```

The command requires the exact `RESTORE` confirmation and prompts for the new Radarr API key without echoing it or putting it in process arguments. It verifies that the URL is Radarr, decrypts into a mode-`0600` temporary file, validates the archive, and uploads it directly to Radarr's restore API. The target's existing Radarr database and settings are replaced; the media files are not copied or changed. Radarr's root-folder paths are restored as recorded, so mount the same paths in the new container or repair them in Radarr after it restarts. Keep the age identity outside `/shared`; never place it in the backup archive or container Compose file.

## Read the Movies and TV summary cards

The Movies and TV pages each show four service-specific summary cards using the same layout. Their counts and disk usage come from Radarr or Sonarr. The metadata-size card shows the encrypted metadata stored for that service's media type—Movies for Radarr and TV for Sonarr—and “All metadata” shows the full shared metadata tree. The total is expected to be the same on both pages; each service-specific subtotal can differ. Sizes count stored files and use binary units (1 GB is 1,024 MB).

On TV, the Sonarr artwork progress message and successful series-load message disappear after every eligible poster has been saved. The metadata worker may continue checking for newly added series without showing a completed-cache message. If posters are still missing, progress remains visible while work continues.

The local plaintext copy stays in node-local `/data`; it is never synchronized. Production deployments must set `RAWARR_LOCAL_PLAINTEXT_CONFIG=false` (the application default) so the configuration exists only as age ciphertext in `/shared/config.aged`. Treat the Radarr key as a secret and do not paste it into logs or documentation.

## Use the roarlay workspace

After signing in at the tailnet URL, use **Overview** for library and shared-folder summaries, **Devices** for approved nodes, routes, peer state, and boneyard scan actions, and **Movies** and **TV** for the selected fossil node's Radarr and Sonarr libraries. The browser prefers a healthy direct Tailscale node route and falls back to roarlay's scoped relay. Its route status reflects a browser probe; a green node or web health page alone does not verify Arr connectivity or synchronization.

Movies and TV offer **Rescan** to request an on-demand metadata refresh on the selected fossil node. This starts its worker; it does not mean every record is already refreshed. The Settings **Export metadata** action downloads a browser-built ZIP of all indexed encrypted movie text records and matching available artwork. It needs the age private ID in the browser, and the ZIP contains decrypted data, so store it privately. The current export builds the archive in browser memory and covers movie metadata only; other media classes, cancellation, and a partial-failure report remain planned. See the [export plan](metadata-export-plan.md).

On an enrolled storage node, scheduled media scans default to daily at 04:00 in the configured `RAWARR_STORAGE_SCAN_TIMEZONE` (UTC if unset). In Devices, each online storage card shows the next scheduled run and live scheduled-scan progress. Owners can change the local time and choose a daily or weekly cadence; the schedule is saved on that node in `/data`. The owner can also request an immediate full scan or bounded movie sample. The scanner reads its node-local `/media` mount and writes encrypted sidecars to `/shared`; a successful scan does not establish a combined cross-node catalog. The Mac mini's first unattended scheduled run was observed writing 403 sidecars before it was intentionally stopped; its fuller scan is scheduled daily at 04:00 EDT. Completion and full-library coverage remain unverified; see the [observation runbook](../runbooks/2026-10-19-observe-unattended-storage-scan.md).

## Scan a local media folder

The media folder must be mounted into the Rawrarr container. The provided Compose overlay mounts one host folder read-only at `/media`:

```sh
export RAWARR_HOST_MEDIA_PATH="/path/to/your/media"
export RAWARR_LIBRARY_NAME="My library"
docker compose -p rawrarr-local -f docker-compose.yml -f docker-compose.media.yml up -d --build
```

Then unlock the age vault, open the dashboard, and choose **Scan folder**. The scanner reads files only; it does not move or rename them. A scan can write age-encrypted metadata sidecars below `/shared/metadata/library/text/` when the shared public recipient is available. Metadata matching uses the configured Radarr integration.

## Configure Syncthing sharing

The Devices panel can use local and remote Syncthing REST APIs to add peers and configure the Rawrarr folder. Keep the local Syncthing API behind a trusted network and provide API keys through protected settings or files. The remote peer API key entered in the form is transient and is cleared after the request.

This form configures the app-data folder; it does not implement user invitations, selective per-user trust, or shared passkey accounts. Back up or test with disposable folders before pointing it at an existing Syncthing folder.

### Replicate a boneyard node's media folder

Media replication is an explicit, separate Syncthing share between two approved boneyard nodes. Mount the verified source media directory into its Syncthing companion at `/media` read-only, and mount an empty destination directory into the second companion at `/media` writable. `docker-compose.syncthing-media.yml` adds this bind to each host's existing Syncthing service; use that host's existing Compose file, `-p rawrarr-local`, and its verified `RAWARR_HOST_MEDIA_PATH`. The Rawrarr storage worker itself keeps `/media` read-only. Never mount media into roarlay or `/shared`.

Choose a source directory whose contents fit on the destination and verify the two Syncthing device IDs with `scripts/share_storage_media.py --docker-container CONTAINER --show-id` on each host. For the current music share, mount the Mini's music directory and the MacBook's `media.ev/music` directory as `/media`. Install the same `.stignore` at each mount root before pairing: `!/[Aa]*`, `!/[Aa]*/**`, then `*`, one pattern per line. This limits the current music tree to its A/a top-level entries; inspect those entries and confirm they are directories before enabling the share. The Mini's read-only bind also needs an existing `.stfolder` directory at its root. Use one matching folder ID, `rawrarr-media`, and `--path /media --include-a-folders` on each host. Run the default dry-run first, then `--apply` on the Mini with `--role source` and `--apply --confirm-empty-destination` on a new MacBook destination with `--role destination`. The source is send-only and the destination receive-only. Existing app-data peer and folder configuration is preserved. The script refuses an existing or overlapping media folder.

The current share also includes three exact album paths: `Mike Jourgensen/Rainy Landfill Music`, `DU Records/Noise Tent 2K Spring Sampler`, and `Longmont Potion Castle/Longmont Potion Castle 15`. To add another album later, run `scripts/add_music_sync_albums.py --root HOST_MUSIC_DIR --album 'Artist/Album'` on both hosts, adding `--source` on the Mini. Review each dry-run and use `--apply` on each host, then scan and verify the bounded transfer. The script adds one exact directory pattern per album before the final `*`; an artist-level allow pattern would include all of that artist's albums.

Check `--status --folder-id ID` on both hosts until they are idle with zero needed files, bytes, and errors. `scripts/hash_media_tree.py` prints a path/content fingerprint for each host folder; compare those outputs before claiming a successful copy. Syncthing sessions are encrypted in transit, but replicated media files are ordinary media bytes on both hosts. Deletions can propagate, so keep separate backups. Adding a Syncthing share does not itself enroll the second Rawrarr boneyard worker; approve that worker in Devices before expecting its media scan or catalog entry.

The current Syncthing connection uses the existing Tailscale sidecars' TCP 22000 Serve forwards and SOCKS5 proxy. The separately enrolled Swarmsail Headscale clients do not use Serve and are not yet attached to Syncthing. Replacing the current path requires a direct Tailscale networking mode for those app containers; preserve the old sidecars until that path and the browser passkey origin have been migrated. [Syncthing proxy settings](https://docs.syncthing.net/users/proxying.html) and [Tailscale userspace networking](https://tailscale.com/docs/concepts/userspace-networking) describe the existing transport behavior.

## Project status

See the [roadmap](../ROADMAP.md) for completed capabilities and planned work. For data placement and security boundaries, see [technical status](technical-status.md) and [data domains](data-domains.md).
