# Rawrarr admin console

## Preview

The operator console is a separate, static container. Its current dashboard uses fictional sample data to preview account, trial, subscription, and operations reporting. It does not connect to PostgreSQL, call an API, load external resources, or contain production account information.

`docker-compose.admin.yml` is a standalone localhost-only preview. It publishes
`http://127.0.0.1:7299` directly from its own container, with no coordinator,
Tailscale sidecar, database, or secret dependencies. It does not configure tailnet
access; do not enable Funnel or add a public reverse-proxy route. The internal
coordinator stays on do3 and is not part of this preview.

The preview intentionally has no sign-in. Before connecting live data, implement and require administrator authentication and authorization in the service, independently of the localhost/Tailscale network boundary. A browser-only gate is not sufficient. Give the reporting service a least-privilege database role and expose only the minimum aggregate/account fields needed; do not reuse the coordinator's write credential. Add audit and access-expiry behavior before production user details are displayed.

## Reporting contract

The first live reporting version should derive account totals from `rawrarr_users` and must distinguish facts from unavailable features. The current schema does not track trial or subscription entitlements, and the documented trial/subscription model remains proposed. Do not infer either status from account age or show sample financial amounts as live data. Integrate those metrics only after billing/trial state is implemented and has a defined source of truth.

The preview has no mutable account controls. Keep future admin actions out of the initial live reporting release unless their authorization, audit trail, confirmation, and recovery behavior are separately designed.

## Local preview

From the repository root, use a separate project after checking that port 7299
and container name `rawrarr-admin` are available:

```sh
docker compose -p rawrarr-admin-preview -f docker-compose.admin.yml up -d --build admin
```

The local URL is `http://127.0.0.1:7299/`. Verify that the admin port appears only
on loopback and that no other services are created. Do not deploy this preview
to a public-facing host until its network path and authentication design are
reviewed.

Existing previews are not automatically moved between projects or namespaces.
Inspect their recorded project/file labels before any recreation and keep their
project name until a separate deliberate migration. A running legacy sidecar
may still own port 7299: stop and resolve its ownership rather than automatically
changing a coordinator, subscriber, or Serve route. Tailnet access supplied by
the old shared namespace is no longer provided by this template. No persistent
data migration is required; the preview has only sample data and no volumes.
For recovery, the old definition and transport overlay are available at Git
published commit `94fda4c`; reverting source alone does not change a running container.
