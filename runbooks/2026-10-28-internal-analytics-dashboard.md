# Internal Umami dashboard access

Issue: [#279](https://github.com/willjasen/rawrarr/issues/279)

## Goal

Make the internal Umami dashboard available at `https://rawrarr.risk-mermaid.ts.net/analytics` through Caddy on the existing internal Tailscale Serve HTTPS route. Keep dashboard access unavailable through the public coordinator.

## Change

- Add an internal-only Caddy service in the internal coordinator's shared Tailscale network namespace. It forwards `/analytics*` to internal Umami and all other requests to the existing coordinator at `127.0.0.1:8080`.
- Build the internal Umami image from upstream commit `559e3c607c23d2c72d9bd4f6481bda428255323c` (Umami 3.4.1) with Docker build arg `BASE_PATH=/analytics`. This is a frontend build setting, so setting it only at runtime is insufficient. Public Umami remains the official image with its default base path.
- Tailscale Serve HTTPS 443 routes to Caddy at `127.0.0.1:8081`; the Caddy fallback preserves roarlay's existing coordinator routes. Remove the temporary 8443 handler created during rollout because clients could not reach that nonstandard port. No public Caddy site, public DNS, PostgreSQL port, or public-origin dashboard route is added.

## Rollout

Publish and merge this change first. On do3, deploy only from the matching GitHub `main` archive. Check free disk space before building; do not prune Docker data. Back up the existing Umami image ID, Compose file, and internal Tailscale Serve configuration. Fetch the public Umami archive at the pinned commit into a new unused build directory, verify its revision, and build the local `rawrarr-umami:559e3c607c23-basepath-analytics` image with BuildKit enabled. Acquire the internal Umami, Tailscale sidecar, and Caddy service deployment locks. Recreate only internal Umami and Caddy with `--no-deps --pull never`; preserve the Umami PostgreSQL volume. Update only the existing HTTPS 443 Serve target to `http://127.0.0.1:8081`, then disable the temporary 8443 handler added in the initial attempt. Preserve TCP handlers and all unrelated Serve configuration. Release locks after verification.

## Verification

- Pending deployment.
- Verify `https://rawrarr.risk-mermaid.ts.net/analytics` loads the Umami login page and static assets/API requests remain under `/analytics`.
- Verify internal `/health` still identifies roarlay and `/analytics/script.js` still serves the tracker.
- Verify the public coordinator continues to serve its tracker but does not serve the Umami dashboard.

## Rollback

Restore internal Tailscale Serve HTTPS 443 to `http://127.0.0.1:8080`, disable only the 8443 handler created during this task, stop/remove only the internal Caddy service, and recreate internal Umami using the original official `ghcr.io/umami-software/umami:3.4.1` image from the base analytics Compose overlay. Preserve both analytics PostgreSQL volumes and the Tailscale state volume. Confirm the HTTPS health route still identifies roarlay.
