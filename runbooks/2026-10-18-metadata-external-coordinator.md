# Keep metadata independent of the coordinator service

## Objective

Configure the metadata node to use a coordinator URL as an external service. The metadata container must not require a coordinator container in the same Compose project.

## Initial state

The primary Compose file set `RAWARR_COORDINATOR_URL` to `http://coordinator:8080` and declared `depends_on: coordinator`. A Compose rebuild therefore tried to start/recreate the local coordinator, which conflicted with the existing Tailscale container owning loopback port 7298. The standalone metadata Compose file already used an HTTPS URL without a coordinator service dependency.

## Changes

- Set the primary metadata service URL from `RAWARR_COORDINATOR_URL`, defaulting to the configured HTTPS coordinator hostname.
- Remove its `depends_on` and local-HTTP override.
- Remove the now-unused `coordinator` DNS alias from the MacBook Tailscale network.
- Update the Tailscale architecture and container deployment docs.

## Verification

- Rendered Compose config shows `https://rawrarr.risk-mermaid.ts.net`, no metadata `depends_on`, and loopback port 7297.
- The metadata container resolved the hostname and fetched the coordinator health page over HTTPS.
- Recreated only metadata with `--no-deps`; container health is healthy and local `/health` returns HTTP 200.
- Tailscale Serve read-back still maps `rawrarr.risk-mermaid.ts.net:443` `/` to `http://127.0.0.1:8080`.

## Final state

Metadata is independently deployable and reaches the coordinator through its configured HTTPS URL. The coordinator may run in the same or another Compose project, but metadata no longer depends on a local Compose service name.
