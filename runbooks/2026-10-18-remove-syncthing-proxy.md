# Remove the Syncthing TCP proxy

> Historical record. The resulting empty `docker-compose.tailscale-serve.yml`
> overlay was retired on 2026-11-03; omit that file from new Compose commands.
> No live Serve handler was changed by its retirement. See the
> [retirement record](2026-11-03-retire-legacy-coordinator-compose.md).

## Objective

Remove the unused `syncthing-proxy` container because live Tailscale Serve forwards TCP 22000 directly to the Syncthing container.

## Initial state

`rawrarr-syncthing-proxy` was running from a standalone Compose project under `~/Library/Application Support/rawrarr/tcp-bridge`. Live Serve status already showed `TCPForward` from port 22000 to `rawrarr-local-syncthing-1:22000`. Tailscale and Syncthing share the `rawrarr-local_default` Docker network.

## Actions and verification

- Removed the proxy service from `docker-compose.tailscale-serve.yml` and removed its now-unused `docker/tcp-bridge` build context.
- Removed the running proxy container with `docker compose ... rm -sf syncthing-proxy`; the active direct Serve forward was left unchanged.
- Removed the obsolete standalone proxy Compose configuration and build file from the local Rawrarr application-support directory.
- Verify `docker ps` has no `rawrarr-syncthing-proxy`, direct Serve status still points TCP 22000 at the Syncthing container, and the coordinator HTTPS health route remains healthy.

## Recovery

If direct peer forwarding stops working, restore the Serve TCP forward to `rawrarr-local-syncthing-1:22000` after checking the Syncthing container's current name and network. The removed proxy definition is preserved in Git history/runbook context, but should not be reintroduced unless the direct route fails.
