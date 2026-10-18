# Plan optional Tailscale access for Rawrarr nodes

## Objective and initial state

Record the near-term plan for optional Tailscale connectivity, a tailnet-scoped stable Rawrarr service name, and full web/API service on storage-capable nodes. Existing architecture described a web-only control node plus a headless scanner; the user clarified that storage nodes should also host the web application. The user wants Tailscale available as an option, not a Rawrarr requirement.

## Plan recorded

- Both web-only and web+storage nodes will run the full Rawrarr web/API service; only the latter mounts `/media`.
- Each Docker deployment shape will be able to add an optional Tailscale sidecar with independent, persistent node-local Tailscale state and a unique node name.
- Per-node MagicDNS/Serve access is the first connectivity milestone. A tailnet-scoped Tailscale Service named `rawrarr` is the later stable entry point across eligible online web nodes.
- Multi-backend advertisement waits until passkey sessions, local PostgreSQL projections, settings unlock, device state, and catalog behavior work no matter which node receives a request. Syncthing is not database replication.
- Tailscale remains optional; local/private-network startup must work without it. Do not enable Funnel/public access or sync Tailscale secrets/state.
- Added the implementation sequence and acceptance checks to [the Tailscale plan](../docs/tailscale-integration-plan.md), and marked it as the next networking milestone in product direction.

## Research and references

Reviewed current official Tailscale documentation for [Services](https://tailscale.com/docs/features/tailscale-services), [Serve](https://tailscale.com/docs/features/tailscale-serve), [Docker Compose](https://tailscale.com/docs/features/containers/docker/how-to/connect-docker-container), [userspace networking](https://tailscale.com/docs/concepts/userspace-networking), and [MagicDNS](https://tailscale.com/docs/features/magicdns). Tailscale Services require tagged service-host identities and current documented client versions; verify requirements again during implementation.

## Verification and remaining work

This was a documentation/planning task; no code, Docker containers, or tailnet settings were changed. Implementation is queued as the next networking milestone. Before enabling multi-host routing, first build full-web storage-node Compose mode and resolve cross-node auth/session and database projection consistency.
