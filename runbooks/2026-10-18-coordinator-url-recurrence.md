# Keep the public coordinator URL on the coordinator

## Objective and initial state

Restore `https://rawrarr.risk-mermaid.ts.net/` to the coordinator and prevent future Serve drift to metadata. Live Serve status pointed HTTPS 443 directly at `http://rawrarr-local-metadata-1:8080`; metadata and coordinator local health checks both returned HTTP 200, confirming the incorrect routing target.

## Actions

Set persistent HTTPS 443 Serve to `http://127.0.0.1:8081`, where the Tailscale namespace coordinator proxy forwards to `coordinator:8080`. Added a project instruction requiring route read-back and external HTTPS health verification after relevant changes, with the safe repair command and a warning not to reset all Serve configuration.

## Verification

Serve status now maps `rawrarr.risk-mermaid.ts.net:443` `/` to `http://127.0.0.1:8081`. HTTPS `/health` serves the coordinator-owned HTML health page. Local metadata remains healthy on 7297; coordinator remains healthy on 7298. No container or data changes were needed.

## Final state

The public tailnet route returns the coordinator page. Instructions now require checking the configured proxy target and its live HTTPS response. Existing unrelated work remains untouched.
