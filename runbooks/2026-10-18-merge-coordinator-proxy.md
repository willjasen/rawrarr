# Run the coordinator in the Tailscale namespace

## Objective

Remove the standalone Caddy coordinator proxy by placing the coordinator process in the MacBook Tailscale container's network namespace, then route Serve directly to the coordinator's loopback listener.

## Initial state

The coordinator listened on container port 8080 and published it as host loopback 7298. Tailscale Serve HTTPS 443 targeted loopback port 8081, where a Caddy sidecar forwarded to `coordinator:8080`. Syncthing peer TCP 22000 was already forwarded directly to the Syncthing container. The coordinator and Tailscale containers share the default Docker network, and PostgreSQL remains a separate service.

## Changes and verification

- Updated the MacBook Tailscale Compose overlay so the coordinator shares the Tailscale network namespace. The Tailscale service now publishes `127.0.0.1:7298` to the coordinator listener and adds the `coordinator` network alias for local containers. Removed both Caddy bridge service definitions; the metadata UI remains on its existing independent loopback port 7297.
- Validated the merged Compose configuration with the protected PostgreSQL password-file path, without reading the secret. Recreated only `tailscale` and `coordinator` with `--no-deps`; PostgreSQL, metadata, Syncthing, and their volumes were left running.
- Changed Tailscale Serve HTTPS 443 to proxy directly to `http://127.0.0.1:8080`. Read-back confirms the HTTPS route uses 8080 and the separate Syncthing TCP forward still targets `rawrarr-local-syncthing-1:22000`.
- Verified the coordinator container is healthy, the shared-namespace loopback and host `127.0.0.1:7298` health pages respond, the Docker `coordinator:8080` alias responds from a disposable same-network check, and local metadata `127.0.0.1:7297/health` returns HTTP 200.
- Verified `https://rawrarr.risk-mermaid.ts.net/health` returns HTTP 200 after removing the orphaned `rawrarr-coordinator-proxy` container. No `metadata-proxy` container was running.
- `docker compose config --quiet` and `git diff --check` passed. No application tests were run; this change only affects deployment networking and documentation.

## Recovery

Restore the prior separate-network Compose layout and the `coordinator-proxy` bridge on namespace port 8081 if the coordinator cannot resolve PostgreSQL, local metadata cannot resolve the `coordinator` alias, host port 7298 is unavailable, or the HTTPS route fails. Restore the Serve HTTPS target to `http://127.0.0.1:8081`. Preserve all PostgreSQL, app, Tailscale, and Syncthing volumes.
