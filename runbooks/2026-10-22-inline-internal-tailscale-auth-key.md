# Embed internal Tailscale auth key in Compose

## Objective and starting state

The internal subscriber installer generated a `tailscale-authkey` file reference. Docker Compose failed on the MacBook because that file was absent. The owner requested that internal-use Compose include the key for now and asked to update the coordinator on the Mac mini. Internal Tailscale nodes should use the `rawrarr-` hostname prefix. The public Headscale path retains its existing secret-file behavior.

## Changes

- Pass the auth key from the browser-only form into the internal subscriber Compose generator. Internal mode embeds the key in `TS_AUTHKEY`, removes the missing secret mount and cleanup wrapper, and prefixes generated Tailscale hostnames with `rawrarr-`.
- Keep Headscale auth keys in the separate local Compose secret file.
- Update the setup instructions and Compose generator documentation to explain the internal mode's inline key and file-permission requirements.
- Rebuild the onboarding bundle and deploy it to the Mac mini `rawrarr-internal-mini` roarlay service.

## Verification

- `npm run build:subscriber-onboarding` completed successfully and regenerated the embedded browser bundle.
- `docker compose config --quiet` succeeded for the user's local `~/rawrarr-node-internal/compose.yml`; the file is mode `0600` and is outside Git.
- `docker compose up -d` started the node Tailscale sidecar and Rawrarr fossil. The sidecar is running. The fossil reports waiting for owner approval, so enrollment is not complete until approved in coordinator Devices.
- Rebuilt the Mac mini's `rawrarr-internal-mini` roarlay container; Docker reports it healthy.
- Read back Tailscale Serve JSON: `rawrarr.risk-mermaid.ts.net:443` `/` proxies to `http://127.0.0.1:8080`. HTTPS `/health` returned HTTP 200 from the live mini service.
- The subscriber bundle and setup page both served over the deployed coordinator route. No auth key was included in verification output.
- Automated script tests were not run in this deployment task.

Never place an actual auth key in this runbook, Git, or logs.

## Recovery

Restore the task-owned Compose generator, onboarding page, and bundle from Git, then rebuild only `rawrarr-internal-mini-roarlay-1`. Preserve its PostgreSQL and Tailscale volumes and the mini's existing subscriber project and native Tailscale Serve handlers.
