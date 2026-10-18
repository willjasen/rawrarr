# Rawrarr project instructions

- Keep all Rust code memory-safe. Do not introduce `unsafe` blocks.
- Keep the service node separate from subscriber nodes. Never place private age identities, passkeys, credentials, media, or subscriber app-data in the service-node or relay code paths.
- Treat `/shared` as synchronized application data, `/media` as node-local media, and `/data` plus PostgreSQL as node-local state. Never commit runtime data, local databases, secrets, `.env` files, or age identities.
- Preserve existing user changes. Before editing, inspect `git status --short --branch`; stage explicit files and review the staged diff before committing.
- For substantive implementation or operations work, create a dated task runbook in `runbooks/` and update its index. Keep design decisions in `docs/DESIGN.md`, architecture in the relevant technical document, and mark unimplemented behavior as planned.
- Maintain tests for meaningful behavior changes under `scripts/tests/`; update their index. Run focused checks for the affected behavior and state clearly what was not verified.
- Keep Rawrarr-specific project work in this repository. Do not import unrelated Arr-services scripts, operational runbooks, service configuration, or media files.

- After UI changes, update the persistent `rawrarr-local` web service on port 7297 and check its health. Preserve its existing data, configuration, and the user’s browser windows.

## Tailnet coordinator URL

- `https://rawrarr.risk-mermaid.ts.net/` and `/health` must route to the coordinator on the MacBook, never to the metadata node. The Tailscale Serve HTTPS 443 proxy target must be `http://127.0.0.1:8081`; in the `rawrarr-macbook-tailscale` network namespace, the `coordinator-proxy` listens on 8081 and forwards to `coordinator:8080`. Port 8080 in that namespace has previously served metadata; do not use it as the Serve target.
- After any Compose, Tailscale sidecar, Serve, or coordinator-proxy change, read back `docker exec rawrarr-macbook-tailscale tailscale serve status --json` and confirm `rawrarr.risk-mermaid.ts.net:443` `/` proxies to `http://127.0.0.1:8081`. Then verify `curl -fsS https://rawrarr.risk-mermaid.ts.net/health` returns `rawrarr coordinator healthy`. A container's healthy status or a local-only health check does not verify this route.
- If Serve status points elsewhere, restore it with `docker exec rawrarr-macbook-tailscale tailscale serve --bg --yes --https=443 http://127.0.0.1:8081`, then repeat both read-back and HTTPS checks. Preserve unrelated Serve handlers and TCP forwards; do not use `tailscale serve reset`.
