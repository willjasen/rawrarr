# Rawrarr user guide

This guide covers features available in the current Docker prototype. For completed and outstanding project work, see the [roadmap](../ROADMAP.md); for data and security constraints, see [technical status](technical-status.md).

## Start Rawrarr with Docker Compose

Requirements: Docker Compose, a local PostgreSQL password file, and a browser with passkey support. OrbStack provides Docker on macOS.

Create a local secret file once; keep it outside the repository and do not overwrite it for an existing database:

```sh
mkdir -p "$HOME/.config/rawrarr"
if [ ! -f "$HOME/.config/rawrarr/postgres-password" ]; then
  umask 077
  openssl rand -base64 32 > "$HOME/.config/rawrarr/postgres-password"
fi
export RAWARR_POSTGRES_PASSWORD_FILE="$HOME/.config/rawrarr/postgres-password"
docker compose up -d --build
```

Open `http://localhost:8080/`. If port 8080 is already in use, set `RAWARR_HOST_PORT` to another free port before starting. Passkey registration also needs matching WebAuthn origin/relying-party settings when using a non-default hostname or port.

Useful commands:

```sh
docker compose logs -f rawrarr
docker compose ps
docker compose down
```

The container logs request method, path, status, and duration by default. Request paths are logged without query parameters, and headers and bodies are never logged. Set `RAWARR_QUIET_LOGS=true` in the Compose environment to suppress routine request lines; server errors remain visible. Successful library scans also log numeric summary counts.

Compose stores the shared Rawrarr data in `~/Library/Application Support/Rawrarr/shared-data` by default, outside the source repository. Open it in Finder with:

```sh
open "$HOME/Library/Application Support/Rawrarr/shared-data"
```

To place it elsewhere, set `RAWARR_SHARED_DATA_PATH` to a persistent host directory before starting Compose. The PostgreSQL and `/data` state remain in Docker-managed volumes.

`docker compose down` stops the services but preserves named volumes and the host shared-data folder. Do not use `docker compose down -v` unless you intend to delete the local PostgreSQL and Rawrarr data volumes.

## First login and age identity

1. On first use, choose a username/display name and select **Create account with a passkey**. The first account becomes the local owner.
2. After passkey sign-in, Rawrarr opens age identity setup. Select **Create ID** to generate an identity. The page shows a shortened preview and a public-recipient fingerprint, then opens a copy pop-up.
3. In the pop-up, select **Copy private identity** and save the key somewhere recoverable, such as a password manager. The clipboard receives only the private key line; the public recipient is not included. The full private identity is not rendered on the page or written to server logs.
4. After the copy succeeds, select **I copied it and saved the private ID somewhere safe.** Enter three consecutive characters starting at character 24 of the whole key (character 5 after the `AGE-SECRET-KEY-PQ-1` header). A correct answer unlocks the encrypted settings and opens the dashboard automatically.
5. To use an existing identity, paste it into the masked field and choose **Copy and verify ID**. In the pop-up, copy and save it, confirm the copy, and enter the requested characters; then select **Unlock vault**. The private identity is not saved by Rawrarr, sent to the relay, or included in `/shared`. Rawrarr cannot recover it if it is lost. After an app restart or new local session, sign in with the passkey and unlock the encrypted config again. An optional Docker secret mount is described in `docker-compose.age-identity.yml`; protect the referenced host identity file accordingly.

The public recipient is safe to share for encryption, but it cannot unlock the config. Never send the private identity to another node or service.

## Configure Radarr

1. Open the Rawrarr dashboard and its **Settings** section.
2. Enter the Radarr base URL and API key, then choose **Save and connect**.
3. Rawrarr saves the settings into `/shared/config.aged` and applies the connection immediately; a Rawrarr restart is not required.
4. Use the connection status or **Movies** view to read Radarr's current movie inventory.

The browser redacts the API key in the readable configuration preview. Treat the Radarr key as a secret and do not paste it into logs or documentation.

## Scan a local media folder

The media folder must be mounted into the Rawrarr container. The provided Compose overlay mounts one host folder read-only at `/media`:

```sh
export RAWARR_HOST_MEDIA_PATH="/path/to/your/media"
export RAWARR_LIBRARY_NAME="My library"
docker compose -f docker-compose.yml -f docker-compose.media.yml up -d --build
```

Then unlock the age vault, open the dashboard, and choose **Scan folder**. The scanner reads files only; it does not move or rename them. A scan can write age-encrypted metadata sidecars below `/shared/metadata/library/` when the shared public recipient is available. Metadata matching uses the configured Radarr integration.

## Configure Syncthing sharing

The Devices panel can use local and remote Syncthing REST APIs to add peers and configure the Rawrarr folder. Keep the local Syncthing API behind a trusted network and provide API keys through protected settings or files. The remote peer API key entered in the form is transient and is cleared after the request.

This configures folder synchronization; it does not implement user invitations, selective per-user trust, shared passkey accounts, or media replication. Back up or test with disposable folders before pointing it at an existing Syncthing folder.

## Project status

See the [roadmap](../ROADMAP.md) for completed capabilities and planned work. For data placement and security boundaries, see [technical status](technical-status.md) and [data domains](data-domains.md).
