# Start the MacBook Tailscale container

## Objective

Start a dedicated Tailscale container on the MacBook Pro and present Tailscale's interactive login flow so the owner can add this container as a tailnet node. Continue with Serve only after enrollment.

## Initial state

- OrbStack Docker context is `orbstack`; Rawrarr coordinator and metadata containers were healthy on loopback ports 7298 and 7297.
- Existing Docker network `rawrarr-local_default` was present; coordinator address on it was `192.168.147.3`.
- The persistent local Tailscale state directory did not exist before this task.

## Actions

- Added `docker-compose.tailscale-macbook.yml`, defining container `rawrarr-macbook-tailscale`, tailnet hostname `rawrarr-macbook`, userspace networking, restart policy, persistent state at `~/Library/Application Support/rawrarr/tailscale/macbook`, and attachment to `rawrarr-local_default`.
- Created the node-local state directory with owner-only permissions.
- Validated the Compose file and started only the Tailscale sidecar under a separate Compose project. Did not restart/rebuild Rawrarr or configure Serve.
- Tailscale printed a one-time interactive login URL. Opened it in the Codex browser; the owner must complete authentication/approval there. The one-time URL is intentionally not recorded in this runbook.

## Verification and remaining work

- The owner completed interactive authorization. Tailscale reports `Running` and assigned the node `rawrarr-macbook.risk-mermaid.ts.net` with a tailnet IP.
- The user plans to rename the node to `rawrarr` in the Tailscale console. Compose `hostname` and `TS_HOSTNAME` have been aligned to `rawrarr` for future restarts; this edit did not restart the container.
- MacBook coordinator, metadata, PostgreSQL, and Syncthing health/status remained unchanged and healthy after sidecar startup.
- Serve remains unconfigured. Next, confirm the console rename, then configure and verify Serve to expose only the coordinator. The Tailscale container and coordinator currently share a Docker network, but Serve supports a loopback proxy target; choose a network-namespace/reverse-proxy design before changing app networking. Do not use Funnel.

## Recovery

Stop only the sidecar with `docker compose -p rawrarr-local-tailscale -f docker-compose.tailscale-macbook.yml stop`. Preserve its state directory to retain node identity. Remove it only after reviewing the exact target and confirming the user no longer needs this tailnet identity.
