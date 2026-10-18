# rawrarr

rawrarr is an early media-library application focused on Radarr first. The root route (`/`) serves a responsive dashboard, `/health` is a service health check, `/api/events` streams dashboard snapshots over server-sent events, `/api/devices` reports scanner devices, `/api/radarr/status` reports Radarr connectivity, `/api/radarr/movies` reads Radarr movie inventory, `/api/library/scan` inventories a configured folder without changing files, and `/api/metadata/movies/search` searches Radarr for movie candidates. The current scanner runs on the web host; remote scanner agents and durable catalog storage are not implemented yet. See [the device architecture](../docs/device-architecture.md).

## Local development

Requires Rust 1.85 or newer.

```sh
cargo run --manifest-path rust-arr/Cargo.toml
open http://127.0.0.1:8080/
curl http://127.0.0.1:8080/health
```

Set `RUST_ARR_LISTEN_ADDR` to change the listen address.

Configure one read-only scan root with `RAWARR_LIBRARY_PATH=/absolute/path/to/media` and optionally set `RAWARR_LIBRARY_NAME`. Restart the service, then use **Scan folder** in the dashboard. The API also supports `GET /api/library` for configuration status and `POST /api/library/scan` to start a scan. Video files are classified as TV when the path contains a `Season N` directory or the filename contains an `SxxEyy` marker; other recognized video files are grouped as movies. Audio, books, subtitles, and disc images are listed separately. This is filename and size inventory only: it does not probe content, rename, move, or delete files. Symbolic links and hidden files/directories are skipped, and traversal has depth and entry caps.

The dashboard opens an `EventSource` connection to `GET /api/events`. It emits a complete dashboard snapshot immediately and then every 30 seconds, including Radarr connectivity, metadata-provider readiness, library setup, and scanner-device state. The browser reconnects automatically if the stream drops; SSE keep-alive comments keep idle connections open.

Set `RAWARR_DEVICE_NAME` to label this host in the Devices panel. The web API currently reports one local scanner device; configuring a separate remote device requires the future agent/pairing implementation described in the device architecture doc.

Configure the optional Radarr integration with `RADARR_URL` (for example, `http://radarr:7878`) and `RADARR_API_KEY`. The key is read from the environment and never returned in responses or error messages. With Compose, set these in an untracked `.env` file in the repository root or in the shell environment. When configured, `GET /api/radarr/status` reads Radarr's `/api/v3/system/status` endpoint. The route returns `503` when Radarr is not configured and `502` if Radarr does not return a successful response.

Radarr can also be configured from the dashboard. First create or import a **post-quantum age identity** in Settings, then unlock the encrypted settings. `PUT /api/settings/radarr` accepts a URL and API key and writes them to `config.age` under `RAWARR_DATA_DIR` (default `./data`), while applying the new connection immediately. After unlocking an existing saved connection, leave the API key blank to preserve it; Rawrarr never returns the key to the UI. `GET` returns only whether the connection is configured, its URL, and whether encrypted settings exist. `DELETE` clears the active connection and encrypted file. If `RADARR_URL` or `RADARR_API_KEY` is set in the environment, those settings can reapply after restart until the environment values are removed. Rawrarr never stores the private identity in its config; after a process restart, import it in the UI to unlock saved settings.

For unattended Docker startup, set `RAWARR_AGE_IDENTITY_PATH` to the identity file path and include the provided secret override: `RAWARR_AGE_IDENTITY_PATH=/secure/path/rawrarr.agekey docker compose -f docker-compose.yml -f docker-compose.age-identity.yml up --build -d`. The override mounts that secret at `/run/secrets/rawrarr_age_identity`; the base Compose file persists ciphertext in the `rawrarr-data` named volume at `/data`. Back up the private identity separately; loss of the identity makes the encrypted settings unrecoverable. Keep the web app bound to localhost because it does not yet have user authentication.

Movie metadata lookup uses Radarr's `/api/v3/movie/lookup` endpoint and the configured Radarr connection. Each movie row offers an on-demand search using a cleaned filename and detected year; rawrarr displays up to five candidates for manual review. No match is automatically applied or saved. Lookup returns `503` until Radarr is configured. The UI includes TMDb's required attribution and an approved logo.

## Download client architecture

`src/download_clients.rs` defines the provider-neutral `DownloadClient` contract and normalized queue, source, receipt, capability, and error types. It is intended to support different transfer protocols, including BitTorrent and Usenet, without exposing provider-specific API models to the rest of rawrarr. The contract currently covers connection checks, queue reads, and submission. It has no concrete adapters and does not contact download clients. Provider configuration, pause/resume/remove actions, and UI integration will be added with their respective implementations. Source URIs may contain credentials or signed parameters and must never be logged.

## Docker

From the repository root:

```sh
docker compose up --build
curl http://127.0.0.1:8080/health
```

The image uses a multi-stage build, includes the age v1.3.2 CLI for hybrid post-quantum encryption, and runs the application as UID 10001. Compose publishes the web UI on host loopback only. The persistent named volume stores encrypted config. `RADARR_URL` and `RADARR_API_KEY` remain available for headless/bootstrap deployments; UI settings require an unlocked age identity.

To scan a host folder from Docker, use the optional read-only media overlay. Set `RAWARR_HOST_MEDIA_PATH` to a folder on the Docker host and run:

```sh
docker compose -f docker-compose.yml -f docker-compose.media.yml up --build
```

The overlay mounts that host folder read-only at `/media` in the container. The app scans `/media` and returns relative paths only. Do not publish the unauthenticated dashboard/API to a LAN or tailnet yet; access control is a later milestone.

## Memory safety policy

The application crate has `#![forbid(unsafe_code)]`, and its Cargo lint configuration rejects unsafe blocks and declarations. This enforces the policy in code written in this crate. Rust dependencies are separate code and may use `unsafe` internally; review their safety notes and keep the dependency set small before expanding the application. A dependency audit is not a proof that all transitive code is memory safe.
