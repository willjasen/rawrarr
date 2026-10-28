# Internal Umami dashboard access

Issue: [#279](https://github.com/willjasen/rawrarr/issues/279)

## Goal

Make the internal Umami dashboard available at `https://rawrarr.risk-mermaid.ts.net/analytics` through Caddy on the internal Tailscale Serve endpoint. Keep dashboard access unavailable through the public coordinator.

## Change

- Add an internal-only Caddy service in the internal coordinator's shared Tailscale network namespace. It forwards `/analytics*` to the internal Umami service and all other requests to the existing coordinator on `127.0.0.1:8080`.
- Configure the internal Umami instance with `BASE_PATH=/analytics`. Leave the public Umami instance's base path unset; its tracker continues through the existing coordinator allow-list proxy.
- Tailscale Serve HTTPS 443 routes to Caddy at `127.0.0.1:8081`. No public Caddy site, public DNS, PostgreSQL port, or public dashboard route is added.

## Rollout

Publish and merge this change first. On do3, deploy only from the matching GitHub `main` archive. Back up the internal Tailscale Serve configuration before changing the HTTPS target. Acquire the internal Tailscale sidecar deployment lock, apply the analytics base path, start the internal Caddy service, then update only the existing HTTPS 443 Serve target to `http://127.0.0.1:8081`. Preserve TCP handlers and all unrelated Serve configuration. Release locks after verification.

## Verification

- Pending deployment.
- Verify `/analytics` loads the Umami login page through the internal HTTPS hostname and static assets/API requests stay under `/analytics`.
- Verify internal `/health` still identifies roarlay and `/analytics/script.js` still serves the tracker.
- Verify the public coordinator continues to serve its tracker but does not serve the Umami dashboard.

## Rollback

Restore the prior internal Tailscale Serve HTTPS target (`http://127.0.0.1:8080`), remove only the internal Caddy service, and unset `RAWARR_ANALYTICS_BASE_PATH` for the internal Umami service. Preserve both analytics PostgreSQL volumes and the Tailscale state volume. Confirm the HTTPS health route still identifies roarlay.
