# Rawrarr admin console

## Preview

The operator console is a separate, static container. Its current dashboard uses fictional sample data to preview account, trial, subscription, and operations reporting. It does not connect to PostgreSQL, call an API, load external resources, or contain production account information.

The optional `docker-compose.admin.yml` overlay attaches the container to the existing MacBook Tailscale network namespace. It binds local access only at `http://127.0.0.1:7299`; it does not publish a public host interface. To reach it over the tailnet, configure a separate Tailscale Serve HTTPS port (for example, `https://rawrarr.risk-mermaid.ts.net:8443/`) to proxy to `http://127.0.0.1:8090`. Tailscale Serve must remain Serve-only; do not enable Funnel or add a public reverse-proxy route. Keep the existing coordinator route on HTTPS 443 unchanged.

The preview intentionally has no sign-in. Before connecting live data, implement and require administrator authentication and authorization in the service, independently of the localhost/Tailscale network boundary. A browser-only gate is not sufficient. Give the reporting service a least-privilege database role and expose only the minimum aggregate/account fields needed; do not reuse the coordinator's write credential. Add audit and access-expiry behavior before production user details are displayed.

## Reporting contract

The first live reporting version should derive account totals from `rawrarr_users` and must distinguish facts from unavailable features. The current schema does not track trial or subscription entitlements, and the documented trial/subscription model remains proposed. Do not infer either status from account age or show sample financial amounts as live data. Integrate those metrics only after billing/trial state is implemented and has a defined source of truth.

The preview has no mutable account controls. Keep future admin actions out of the initial live reporting release unless their authorization, audit trail, confirmation, and recovery behavior are separately designed.

## Local preview

With the existing `rawrarr-local` Compose project and Tailscale overlay configured:

```sh
docker compose -p rawrarr-local -f docker-compose.yml -f docker-compose.tailscale-macbook.yml -f docker-compose.admin.yml up -d --build admin
```

The local URL is `http://127.0.0.1:7299/`. The Tailscale Serve HTTPS endpoint should use a separate port so the existing `:443` coordinator route is preserved. Verify that the admin port appears only on loopback, the container has no database dependency, and the Serve status has no Funnel/public handler. Do not deploy this preview to a public-facing host until its network path and authentication design are reviewed.
