# Local media folder scan

## Objective and initial state

Add a usable, read-only local media inventory to the rawrarr dashboard, with a Docker deployment option. The dashboard already served on `127.0.0.1:8080`; Radarr status was configurable, but no local scan was available. Related context: [dashboard UI](2026-10-17-radarr-dashboard-ui.md), [first Radarr integration](2026-10-17-radarr-first-integration.md), and [download client abstraction](2026-10-17-download-client-abstraction.md).

## Actions

- Added one explicitly configured library root via `RAWARR_LIBRARY_PATH`, with optional display name. The scanner walks read-only, skips hidden entries and symlinks, emits relative paths, file sizes, categories, counts, access issues, and truncation status. It caps traversal at depth 64, 250,000 entries, and 10,000 media files.
- Added `GET /api/library` and `POST /api/library/scan`, and connected these to a scan panel in the dashboard.
- Added Docker Compose environment configuration and a read-only `docker-compose.media.yml` bind-mount overlay. Host port 8080 is loopback-bound; the container listens on its Docker interface.
- Kept the local development server unconfigured so it does not inventory an arbitrary or real media directory.
- Checked `https://radarr.risk-mermaid.ts.net/api/v3/system/status` without credentials; it returned HTTP 401, confirming endpoint reachability and that authentication is required. A credentialed integration check remains pending.

## Verification

- `cargo fmt` and `cargo build --release --locked` passed with Rust 1.85.1.
- A disposable fixture scan returned one item each for book, movie, music, other, subtitle, and television; paths were relative, a symlink target was excluded, and no scan issues/truncation occurred.
- A disposable local Radarr HTTP mock verified `/api/v3/system/status`, the `X-Api-Key` header, and the returned status fields through rawrarr.
- `docker compose config` passed for the default deployment and the media overlay using `/tmp` as a placeholder host path.
- Local HTTP smoke checks: `/health` returned `200 ok`; `/api/library` returned `configured:false`; `/api/library/scan` returned the expected `503` while no folder was configured. Dashboard browser showed the new scan panel and disabled button until configuration.
- `git diff --check` passed. Docker image build was not checked.

## Final state and remaining work

The local scan feature is available and the development dashboard remains running at `http://127.0.0.1:8080/`. To inventory a real folder, configure `RAWARR_LIBRARY_PATH` locally or set `RAWARR_HOST_MEDIA_PATH` for the Docker overlay, then restart rawrarr. The current media volume is not mounted in this workspace, so no real library was read. Radarr access is reachable but awaits locally supplied credentials for an authenticated status check.
