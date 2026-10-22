# Deploy subscriber setup changes to the internal coordinator

## Objective and starting state

Deploy the committed `/setup/node` section-collapse, Compose clipboard, and node-wait navigation changes to the internal coordinator on `brandons-mac-mini`. The isolated `rawrarr-internal-mini` project was already running healthy PostgreSQL and roarlay services with a working Tailscale Serve route. The prior implementation is recorded in [open Devices when the subscriber node connects](2026-10-22-redirect-after-node-online.md).

## Work and verification

- Confirmed the existing project and Compose source on the mini. Claimed `~/Library/Application Support/Rawrarr/.container-rebuild-roarlay.lock` atomically.
- Backed up the two prior setup source files to `~/Library/Application Support/rawrarr/internal-coordinator/backups/pre-8fc1f89/`, then copied only the committed setup HTML and JavaScript assets into the existing build source. Their SHA-256 hashes matched the task checkout.
- A first Compose attempt ran on the workstation and stopped before changing a service because the mini source path was not local. Re-ran the build over SSH on the mini.
- Updated only `roarlay` in project `rawrarr-internal-mini` with `docker compose ... up -d --no-deps --build roarlay`. PostgreSQL and the Tailscale sidecar were not restarted. The resulting image is `sha256:321ccd13f1f5d0a2f5287d2d5224ee6c55f0b5123d18ff476aa8441dae3aee9b`.
- Verified roarlay and PostgreSQL are healthy; roarlay has zero restarts. Mini loopback `/health` and tailnet HTTPS `/health` returned HTTP 200. Tailscale Serve JSON maps HTTPS `/` to `http://127.0.0.1:8080`.
- The served onboarding JavaScript hash matches the committed asset. An authenticated Chrome tab displayed the Compose copy control and waiting status text; `/#devices` rendered the Devices view. The original setup tab was left untouched.
- Released the roarlay deployment lock and confirmed it was gone. The browser automation test remains unavailable because its configured Playwright module is missing; the remote Docker build compiled the Rust service successfully.

## Recovery and remaining work

To restore the previous setup page source, copy the saved HTML and JavaScript files from the backup directory over the corresponding files under `internal-coordinator/source`, then rebuild only roarlay under the same lock and verify health and Serve routing. No database or node state changed during this deployment.
