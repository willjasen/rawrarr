# Route Rawrarr HTTPS to the coordinator

## Objective and initial state

Ensure `https://rawrarr.risk-mermaid.ts.net/` reaches the coordinator rather than MacBook metadata. Initial live Serve configuration forwarded HTTPS 443 to loopback 8080 in the Tailscale namespace; the misleadingly named `rawrarr-coordinator-proxy` forwarded that port to `metadata:8080`. Coordinator and metadata were healthy on host loopback 7298 and 7297 respectively.

Existing uncommitted implementation and deployment changes are unrelated and preserved. The existing Serve overlay already declares the intended coordinator bridge on 8081 to `coordinator:8080`; its running container was stale. No remote is configured.

Related context: [role separation](2026-10-18-central-coordinator-storage-metadata.md), [Tailscale setup](2026-10-18-start-macbook-tailscale-container.md).

## Actions

Reconcile only `coordinator-proxy` using the existing two Tailscale Compose files under project `rawrarr-local`, with `--no-deps`. Set persistent background Tailscale HTTPS Serve to `http://127.0.0.1:8081`. Preserve subscriber services, state, volumes, and local port 7297.

## Verification

- Serve read-back: HTTPS 443 `/` proxies to `http://127.0.0.1:8081`; no Funnel configuration present.
- Recreated proxy command read-back: Caddy listens on 8081 and forwards to `coordinator:8080`.
- Verified with normal certificate validation from the MacBook: HTTPS `/health` returns HTTP 200 and exactly `rawrarr coordinator healthy`.
- HTTPS `/` and `/api/auth/status` return HTTP 401 without a node credential; the metadata UI is no longer served at this origin.
- Local coordinator health on 7298 returns the same coordinator response. Metadata on 7297 returns HTTP 200; both app containers, PostgreSQL, and Syncthing remain healthy.
- No second-device or restart test was performed. Serve runs in persistent background mode; the proxy uses the existing Compose restart policy.
- Documentation whitespace checks passed. No application source changes or rebuilds were needed.

## Final state

Requested origin now reaches the coordinator. Only this task's runbook and index entry are committed; extensive pre-existing changes remain untouched. Branch `main`, no Git remote/upstream configured, so the commit remains local.

## Recovery

Reconcile the coordinator bridge from the existing Compose overlay and reapply Serve to port 8081 if routing drifts. The old port 8080 target exposed metadata and must not be restored as a coordinator route. No application data migration or deletion is involved.
