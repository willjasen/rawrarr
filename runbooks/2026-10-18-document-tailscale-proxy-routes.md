# Document Tailscale coordinator and Syncthing routes

## Objective

Clarify the local MacBook route architecture after removing the unnecessary Syncthing TCP bridge, and explain why the coordinator's loopback bridge remains.

## Initial state

The live Tailscale Serve configuration had HTTPS 443 targeting `http://127.0.0.1:8081` and a raw TCP forward on 22000 directly to `rawrarr-local-syncthing-1:22000`. The coordinator proxy listens on 8081 in the Tailscale network namespace and forwards to the separate coordinator container on 8080. The Syncthing GUI is loopback-bound on host port 8385. Project notes described these paths inconsistently and did not explain the Serve loopback requirement.

## Actions

- Updated the project instructions, design notes, Tailscale integration plan, and container deployment guide to distinguish HTTPS reverse proxying from raw TCP forwarding.
- Recorded that the HTTPS route keeps its loopback Caddy bridge, the Syncthing peer route needs no proxy container, and the Syncthing GUI has no tailnet URL by default.
- Added links to the current deployment and route runbook for operational context.

## Verification

Reviewed the installed `tailscale serve --help`, the [official Serve CLI reference](https://tailscale.com/docs/reference/tailscale-cli/serve), live Serve JSON, Compose definitions, and container networking. The official reference documents HTTP reverse-proxy targets as loopback-only. Live HTTPS `/health` returned HTTP 200 during review. Documentation whitespace validation passed.

## Final state

Documentation matches the current direct Syncthing TCP forward and coordinator HTTPS loopback bridge. No runtime services or Tailscale settings changed for this documentation task.
