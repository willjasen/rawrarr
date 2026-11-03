# Expose SwarmSail on do3 through Tailscale Serve

## Objective and issue

Make the existing SwarmSail UI on do3 available to the operator over the `risk-mermaid` tailnet and open it in the browser. Tracked by [issue #301](https://github.com/willjasen/rawrarr/issues/301).

## Initial state

- The public Rawrarr Compose project on do3 is `rawrarr-roarlay-prod`, using `/opt/rawrarr-roarlay-prod/docker-compose.roarlay-prod.yml`.
- Its Headscale service was running. The existing SwarmSail service was stopped, though its previous logs showed it listening on port 8787. The Compose file maps that service to host loopback at `127.0.0.1:18787` and attaches it to the `rawrarr-roarlay-prod-control` network.
- The public project's Tailscale sidecar is joined to a separate Headscale network with DNS suffix `tailnet.rawrarr.willjasen.net`. The desktop browser uses the `risk-mermaid` tailnet instead.
- The host's `tailscale serve status --json` already contained TCP forwards on ports 17300 and 17301. Preserve those forwards.

## Actions

- Started only `swarmsail` in the existing Compose project with `docker compose ... up -d --no-deps swarmsail`; Headscale was already running. Claimed `/opt/rawrarr-roarlay-prod/.container-rebuild-swarmsail.lock` atomically before starting it.
- Initially added HTTP Serve on the public project's Headscale sidecar. Its URL was not resolvable from the desktop browser because that browser was not connected to this separate Headscale network; the two Mac peers listed in that network were offline.
- Added HTTPS Serve on the do3 host's `risk-mermaid` Tailscale node, proxying to the existing loopback mapping `http://127.0.0.1:18787`. The resulting operator URL is `https://rawrarr-roarlay.risk-mermaid.ts.net/`.
- Removed the temporary HTTP handler from the separate Headscale sidecar after verifying the operator-tailnet route, leaving SwarmSail served only on the operator's `risk-mermaid` node.
- Kept the existing TCP Serve forwards on ports 17300 and 17301.

## Verification

- `curl -fsS -o /dev/null -w 'HTTP %{http_code}' http://127.0.0.1:18787/` returned HTTP 200.
- Host `tailscale serve status --json` showed HTTPS 443 at `rawrarr-roarlay.risk-mermaid.ts.net:443` proxying to `http://127.0.0.1:18787`, with the existing TCP forwards unchanged.
- Opened the HTTPS URL in the Codex browser. The SwarmSail page loaded with its Organizations, Connect organizations, New device, Share device, and ACL rule sections.
- The separate Headscale-sidecar URL was not reachable from the desktop browser and is not the operator URL.

## Recovery and remaining work

- Remove only the host HTTPS handler with `tailscale serve --https=443 off` if SwarmSail should no longer be available on the `risk-mermaid` tailnet. The existing TCP forwards remain.
- Stop the app without removing its container or data with `docker compose -p rawrarr-roarlay-prod -f /opt/rawrarr-roarlay-prod/docker-compose.roarlay-prod.yml stop swarmsail`.
- No public DNS, public Caddy route, Headscale membership, or subscriber ACL was changed.
