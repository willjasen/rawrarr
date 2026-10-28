# Internal Umami dashboard access

Issue: [#279](https://github.com/willjasen/rawrarr/issues/279)

## Goal

Make the internal Umami dashboard available at `https://rawrarr.risk-mermaid.ts.net:8443/` through Caddy on a separate internal Tailscale Serve HTTPS port. Keep dashboard access unavailable through the public coordinator.

## Change

- Add an internal-only Caddy service in the internal coordinator's shared Tailscale network namespace. It forwards its root path to the internal Umami service.
- Preserve Umami's default root path so the existing coordinator tracker requests and dashboard static assets both work without a custom Umami build.
- Add a separate Tailscale Serve HTTPS 8443 route to Caddy at `127.0.0.1:8081`. Preserve existing HTTPS 443 routing to roarlay. No public Caddy site, public DNS, PostgreSQL port, or public-origin dashboard route is added.

## Rollout

Publish and merge this change first. On do3, deploy only from the matching GitHub `main` archive. Back up the internal Tailscale Serve configuration before adding a port. Acquire the internal Tailscale sidecar and Caddy service deployment locks, start the internal Caddy service, then add HTTPS 8443 pointing to `http://127.0.0.1:8081`. Preserve the existing HTTPS 443 route, TCP handlers, and all unrelated Serve configuration. No Umami restart or data migration is required. Release locks after verification.

## Verification

- Pending deployment.
- Verify `https://rawrarr.risk-mermaid.ts.net:8443/` loads the Umami login page and its static assets/API requests work at the root path.
- Verify internal `/health` still identifies roarlay and `/analytics/script.js` still serves the tracker.
- Verify the public coordinator continues to serve its tracker but does not serve the Umami dashboard.

## Rollback

Disable only the new Tailscale Serve HTTPS 8443 handler and remove the internal Caddy service. Preserve the existing HTTPS 443 handler, both analytics PostgreSQL volumes, and the Tailscale state volume. Confirm the HTTPS health route still identifies roarlay.
