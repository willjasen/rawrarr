# Group the MacBook Tailscale container with rawrarr-local

## Objective

Place the MacBook Tailscale container in the existing `rawrarr-local` Docker Compose project group without restarting Rawrarr services or losing the pending Tailscale identity state.

## Initial state

- `rawrarr-macbook-tailscale` was running under Compose project `rawrarr-local-tailscale`.
- Its state was bind-mounted from `~/Library/Application Support/rawrarr/tailscale/macbook`.
- The coordinator and metadata containers were healthy in project `rawrarr-local`.

## Actions and verification

- Stopped and removed only the Tailscale container using its old Compose project. The bind-mounted state directory was preserved.
- Recreated it from `docker-compose.tailscale-macbook.yml` with project name `rawrarr-local`.
- Confirmed the container label now reports `com.docker.compose.project=rawrarr-local`, it is running, and the coordinator and metadata remain healthy.
- The restart refreshed the interactive Tailscale login URL. The current URL was opened in the Codex browser; the owner still needs to authorize `rawrarr-macbook`.

## Remaining work

After enrollment, configure Tailscale Serve to the coordinator and verify tailnet access. Tailscale container configuration should continue to use:

```sh
RAWARR_TAILSCALE_STATE_PATH="$HOME/Library/Application Support/rawrarr/tailscale/macbook" \
  docker compose -p rawrarr-local -f docker-compose.tailscale-macbook.yml up -d
```

When operating this one-service Compose file under the shared project name, do not pass `--remove-orphans`; the other `rawrarr-local` services are intentionally described in different Compose files.
