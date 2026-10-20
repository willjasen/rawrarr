# Admin console preview

## Scope

Add a separate, static admin-console container to preview account, trial, subscription, and operations reporting. The page uses fictional sample data only. It does not access production PostgreSQL, the coordinator API, subscriber data, or any billing provider. Admin login is planned but intentionally not implemented for this preview; it is a required gate before any live account data is connected.

## Network boundary

`docker-compose.admin.yml` attaches the web container to the existing Tailscale sidecar namespace and publishes host port 7299 only on `127.0.0.1`. For tailnet browsing, use a distinct Tailscale Serve HTTPS port proxying to `http://127.0.0.1:8090`; preserve the existing coordinator `:443` handler. Do not enable Funnel, a public reverse proxy, or a non-loopback host port.

For visual review, a standalone sample-only container was started on loopback at `http://127.0.0.1:7299/`. The optional Compose service was not started; no Tailscale Serve state or production services were changed.

## Verification

- Run `node --test scripts/tests/test_rawrarr_admin_preview.mjs`.
- Render the optional Compose overlay and inspect its ports, network mode, and dependencies before deployment.
- After deployment, verify `http://127.0.0.1:7299/health`, `http://127.0.0.1:7299/`, the Tailscale HTTPS endpoint, and the live Serve JSON. Confirm `:443` remains routed to roarlay and the admin HTTPS port is Serve-only, without Funnel/public exposure.
- Before connecting live reporting, implement server-side admin authentication and authorization, a least-privilege read-only database role, and audit controls.
