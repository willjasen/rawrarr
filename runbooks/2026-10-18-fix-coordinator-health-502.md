# Fix coordinator health 502

## Objective and initial state

Restore `https://rawrarr.risk-mermaid.ts.net/health`. The coordinator container was healthy and local port 7298 returned HTTP 200. Tailscale Serve inspection showed port 443 forwarding to loopback 8080, while the coordinator Caddy bridge listens on 8081 and forwards to `coordinator:8080`. Port 8443 already pointed to 8081. Metadata remains on 7297.

## Action

Updated persistent Tailscale Serve HTTPS port 443 to `http://127.0.0.1:8081`. Did not restart containers or modify data.

## Verification

Serve status now maps HTTPS 443 to 8081; HTTPS `/health` returns HTTP 200. The deployed coordinator health test passes against the public tailnet URL, including both page routes and protected API behavior.

## Final state

Coordinator health page is reachable through the requested HTTPS endpoint. No container state or application data changed. Git remote is not configured; task runbook saved locally.
