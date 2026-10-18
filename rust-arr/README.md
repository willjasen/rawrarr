# rawrarr

Rawrarr is an early unified media automation prototype, starting with Radarr integration. Its intended direction includes Prowlarr-class discovery alongside library management, fulfillment, and remote devices; indexer management and provider search are not implemented yet. The root route (`/`) serves the authenticated dashboard, `/auth` starts passkey onboarding/sign-in, `/setup/age` prompts for age identity creation/import, `/health` is a service health check, `/api/events` streams dashboard snapshots over server-sent events, `/api/devices` reports scanner devices, `/api/configuration` shows the redacted decrypted configuration and its age-encryption verification state, `/api/radarr/status` reports Radarr connectivity, `/api/radarr/movies` reads Radarr movie inventory, `/api/library/scan` inventories a configured folder and writes age-encrypted JSON sidecars into shared data, and `/api/metadata/movies/search` searches Radarr for movie candidates. See [product direction](../docs/product-direction.md) and [the device architecture](../docs/device-architecture.md).

## Passkey sign-in and first run

PostgreSQL is required. Rawrarr applies SQL migrations at startup and stores user records, WebAuthn passkeys, short-lived ceremony state, and hashed session tokens there. Passkeys are validated by `webauthn-rs`; the browser receives only an opaque HttpOnly session cookie. The first visitor creates the owner account with a passkey. Later users cannot self-register yet; account invitation and multi-user account management are future work. After passkey sign-in, Rawrarr directs the user to the separate age identity setup screen, then opens the existing dashboard.

WebAuthn requires HTTPS or a browser-recognized localhost origin. Configure `RAWARR_WEBAUTHN_RP_ID` to the host name users enter and `RAWARR_WEBAUTHN_ORIGIN` to the full origin (scheme, host, and port). For the default local Compose deployment, open `http://localhost:8080` (not `127.0.0.1`) so the configured origin matches. For a reverse proxy, use its public HTTPS origin and domain. The age identity is not stored in PostgreSQL; it remains client-provided and unlocks the existing encrypted Rawrarr configuration.

## Local development

Requires Rust 1.85 or newer.

Start PostgreSQL and set `RAWARR_DATABASE_URL` (or `RAWARR_PG_HOST`, `RAWARR_PG_USER`, `RAWARR_PG_DATABASE`, and `RAWARR_PG_PASSWORD` / `RAWARR_PG_PASSWORD_FILE`) before running Rawrarr:

```sh
cargo run --manifest-path rust-arr/Cargo.toml
open http://localhost:8080/
curl http://localhost:8080/health
```

Set `RUST_ARR_LISTEN_ADDR` to change the listen address.

Configure a scan root on the device that can read the media with `RAWARR_LIBRARY_PATH=/absolute/path/to/media`; optionally set `RAWARR_LIBRARY_NAME` and a stable unique `RAWARR_DEVICE_ID`. If omitted, a UUID is created once in the node-local `RAWARR_DATA_DIR/device-id` file (default `data/device-id`). The dashboard's **Scan folder** action requires an unlocked post-quantum age identity. The API supports `GET /api/library` for configuration status and `POST /api/library/scan` to scan from the web process. A separate storage-only process can run `rawrarr storage-scan-once`; it needs `RAWARR_LIBRARY_PATH`, `RAWARR_SHARED_DATA_DIR`, and a local data directory, but no PostgreSQL, web server, passkey session, or private age identity. Unlocking the age identity on a web node writes the public recipient to `/shared/age-recipient`; the storage process uses that public key to encrypt metadata. The private identity remains in web-process memory when unlocked and is never written to the shared folder. Along with the UI inventory, each recognized file produces a JSON sidecar encrypted as `.json.aged` under `<RAWARR_SHARED_DATA_DIR>/metadata/library/`; Syncthing can replicate those ciphertext sidecars to the web node. The encrypted payload contains schema version, device ID, library name, relative path, media kind, byte size, and file modification time. The filename is a deterministic hash of device ID, library name, and relative path, so media paths are not exposed in shared-folder filenames. Video files are classified as TV when the path contains a `Season N` directory or the filename contains an `SxxEyy` marker; other recognized video files are grouped as movies. Audio, books, subtitles, and disc images are listed separately. Scanning does not rename, move, delete, or probe media contents. Symbolic links and hidden files/directories are skipped, and traversal has depth and entry caps.

For a one-shot Docker storage scan, mount its media root read-only at `/media`, the Syncthing shared Rawrarr folder read-write at `/shared`, and a node-local persistent volume at `/data`, then run the image with the `storage-scan-once` argument. Example: `docker run --rm -v /path/to/media:/media:ro -v /path/to/rawrarr-shared:/shared -v rawrarr-storage-data:/data -e RAWARR_LIBRARY_PATH=/media -e RAWARR_SHARED_DATA_DIR=/shared -e RAWARR_DATA_DIR=/data rawrarr:local storage-scan-once`. This remains a scan utility. The planned persistent storage-node deployment runs the full web/API service plus its own `/media`; see the [Tailscale integration plan](../docs/tailscale-integration-plan.md).

The dashboard opens an `EventSource` connection to `GET /api/events`. It emits a complete dashboard snapshot immediately and then every 30 seconds, including Radarr connectivity, metadata-provider readiness, library setup, and scanner-device state. The browser reconnects automatically if the stream drops; SSE keep-alive comments keep idle connections open.

Set `RAWARR_DEVICE_NAME` to label this host in the Devices panel. Scanner devices and Syncthing storage peers are separate roles; the web API reports local scanner state and exposes Syncthing peer status at `GET /api/syncthing/status`.

### Syncthing storage peers

The Devices panel can configure a web node and a storage node to share a folder. The Rawrarr web container needs access to its local Syncthing REST API through `RAWARR_SYNCTHING_API_URL` and either `RAWARR_SYNCTHING_API_KEY` or `RAWARR_SYNCTHING_API_KEY_FILE`. In Compose, provide these values through the environment; use a mounted secret file for the key when possible. The UI asks for the storage node's REST URL/key, the folder paths on both nodes, and network addresses each node can use to reach the other on Syncthing's port 22000. Use HTTPS or a trusted private network for the remote REST API, and do not publish Syncthing's REST API publicly.

Rawrarr adds each peer to the other's Syncthing config and configures the web folder as send-only and the storage folder as receive-only. Syncthing handles the peer protocol, TLS, and file transfer. Rawrarr does not age-encrypt arbitrary folder contents; files that must remain age encrypted need to be written as `.aged` ciphertext before syncing. The OrbStack integration test runs two isolated `syncthing/syncthing:2.1.5` containers and checks the received file's SHA-256:

```sh
python3 scripts/tests/test_rawrarr_syncthing_orbstack.py
```

Configure the optional Radarr integration in the dashboard after unlocking the age identity. Rawrarr stores the URL and API key only in the age-encrypted shared config; it does not accept plaintext Radarr credentials from environment variables. When configured, `GET /api/radarr/status` reads Radarr's `/api/v3/system/status` endpoint. The route returns `503` when Radarr is not configured and `502` if Radarr does not return a successful response.

First create or import a **post-quantum age identity** in Settings, then unlock the encrypted settings. `PUT /api/settings/radarr` accepts a URL and API key and writes them as age ciphertext to `config.aged` under `RAWARR_SHARED_DATA_DIR` (default `data/shared`), while applying the new connection immediately. After unlocking an existing saved connection, leave the API key blank to preserve it; Rawrarr never returns the key to the UI. `GET` returns only whether the connection is configured, its URL, and whether encrypted settings exist. `DELETE` clears the active connection and encrypted file. Rawrarr never stores the private identity in its config; after a process restart, import it in the UI to unlock saved settings.

The stable data domains are documented in [`docs/data-domains.md`](../docs/data-domains.md). For Docker, Rawrarr keeps node-local app data at `/data`, bind-mounts the host Syncthing folder at `/shared`, and exposes `/metadata` as an alias of `/shared/metadata` on every node. Set `RAWARR_SHARED_DATA_PATH` to the host folder configured as this node's Rawrarr Syncthing folder; it defaults to `./shared-data`, which is only a local directory until Syncthing is configured to sync it. A storage node additionally mounts its media root read-only at `/media`; web nodes do not need that domain. `config.aged` stays under `/shared`, while encrypted media sidecars and artwork are written through `/metadata`. PostgreSQL, passkey registrations, sessions, and local-only app data remain in node-local volumes today. Sharing portable user/passkey records across nodes is a product requirement, but still needs an implementation and a secure passkey-first bootstrap design for a new node. Back up the age identity separately; loss of the identity makes encrypted settings unrecoverable. The web app has passkey authentication and Compose publishes it on host loopback by default.

Movie metadata lookup uses Radarr's `/api/v3/movie/lookup` endpoint and the configured Radarr connection. Each movie row offers an on-demand search using a cleaned filename and detected year; rawrarr displays up to five candidates for manual review. Poster art is fetched only from TMDb's image host, saved as age-encrypted `.aged` files under `/shared/metadata/movies/`, and served from the local API only while the age vault is unlocked. Poster cache writes are capped at 8 MiB and use shared-folder-readable permissions. No match is automatically applied or saved. Lookup returns `503` until Radarr is configured. The UI includes TMDb's required attribution and an approved logo.

## Download client architecture

`src/download_clients.rs` defines the provider-neutral `DownloadClient` contract and normalized queue, source, receipt, capability, and error types. It is intended to support different transfer protocols, including BitTorrent and Usenet, without exposing provider-specific API models to the rest of rawrarr. The contract currently covers connection checks, queue reads, and submission. It has no concrete adapters and does not contact download clients. Provider configuration, pause/resume/remove actions, and UI integration will be added with their respective implementations. Source URIs may contain credentials or signed parameters and must never be logged.

## Docker

Create a protected password file outside the repository and point `RAWARR_POSTGRES_PASSWORD_FILE` to it. For example, create `/secure/path/rawrarr-postgres-password` with a random value, set its permissions to owner-only, and export `RAWARR_POSTGRES_PASSWORD_FILE=/secure/path/rawrarr-postgres-password`. From the repository root:

```sh
docker compose up --build
curl http://localhost:8080/health
```

Set `RAWARR_HOST_PORT` to choose another host port while keeping the container port at 8080 (for example, `RAWARR_HOST_PORT=8083`).

The image uses a multi-stage build, includes the age v1.3.2 CLI for hybrid post-quantum encryption, and runs the application as UID 10001. Compose runs PostgreSQL on the private Compose network without publishing its port, persists the database and local app data in separate Docker volumes, and mounts the supplied password file as a Docker secret. It publishes the web UI on host loopback only. The host's Rawrarr Syncthing folder is bind-mounted at `/shared`; set `RAWARR_SHARED_DATA_PATH` to the folder used by Syncthing. Set `RAWARR_WEBAUTHN_RP_ID` and `RAWARR_WEBAUTHN_ORIGIN` in the environment when using a domain or non-default origin.

To scan a host folder from Docker, use the optional read-only media overlay. Set `RAWARR_HOST_MEDIA_PATH` to a folder on the Docker host and run:

```sh
docker compose -f docker-compose.yml -f docker-compose.media.yml up --build
```

The overlay mounts that host folder read-only at `/media` in the container. The app scans `/media` and returns relative paths only. Do not expose the dashboard directly to the public internet; deploy behind HTTPS and configure the WebAuthn origin and relying-party ID for the chosen domain.

## Memory safety policy

The application crate has `#![forbid(unsafe_code)]`, and its Cargo lint configuration rejects unsafe blocks and declarations. This enforces the policy in code written in this crate. Rust dependencies are separate code and may use `unsafe` internally; review their safety notes and keep the dependency set small before expanding the application. A dependency audit is not a proof that all transitive code is memory safe.
