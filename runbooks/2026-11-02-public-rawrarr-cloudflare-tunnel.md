# Publish public Rawrarr at rawrarr.net through Cloudflare Tunnel

Issue: [#320](https://github.com/willjasen/rawrarr/issues/320)

## Goal

Serve the existing public Rawrarr coordinator at `https://rawrarr.net/` through Cloudflare Tunnel and allow only Will's current public IP during early access. The access control must cover every public route to the application so the existing hostname or origin address cannot bypass the allowlist.

## Findings

- The public coordinator runs on do3 as `rawrarr-roarlay-prod-roarlay`; the existing public endpoint is `https://rawrarr.willjasen.net/` and its `/health` route returned HTTP 200.
- `rawrarr.net` initially had no web record. Its apex TXT record was `v=spf1 -all` and has been preserved.
- do3's Caddy currently publishes `104.248.184.91:443` directly and serves the public Rawrarr app as the default handler for `rawrarr.willjasen.net`. It also carries Headscale routes, so removing Caddy's public port would disrupt unrelated service routes.
- A Cloudflare Tunnel token file exists at `/opt/synkpieces/secrets/cloudflared-token` on do3. The `cloudflared` service that uses it is profile-gated and stopped. The token value is a secret and must never be read into logs, chat, or this repository.
- The API token supplied at `/opt/rawrarr-roarlay-prod/secrets/cloudflare-api-token` was active and could manage the `rawrarr.net` zone. Its file mode was tightened from 0644 to 0400 (`root:root`). Its value was never printed or stored in Git.
- Current client egress IPv4 was observed during the task but is intentionally not recorded in Git.

## Cloudflare configuration

Cloudflare's current recommendation is a WAF custom rule for IP-based blocking, rather than a Page Rule. The active **Block** rule uses this expression with the currently approved client IPv4 substituted:

```text
http.host eq "rawrarr.net" and ip.src ne <verified-client-ip>
```

The expression is scoped to `rawrarr.net`. If `www.rawrarr.net` is later added as a public alias, include it in both the tunnel route and the rule expression. The rule contains a single current IPv4; update it if the client's ISP or VPN egress changes.

The API token was stored outside the repository at `/opt/rawrarr-roarlay-prod/secrets/cloudflare-api-token`, owned by root with mode 0400. Do not send the token in chat or store it in Git. The token must be scoped to the account containing the tunnel and the `rawrarr.net` zone, with Account > Cloudflare Tunnel > Edit, Zone > DNS > Edit, and Zone > WAF > Edit permissions. Cloudflare's tunnel setup documentation lists the Tunnel Edit and DNS Edit permissions; custom-rule deployment requires Zone WAF Write.

The deployed Cloudflare Tunnel is `rawrarr-net` (`b0b4e8a6-75e1-44cb-8a2d-c308075f5e4b`). Its remote ingress maps `rawrarr.net` to `http://tailscale:8080`, and its proxied apex CNAME points to the tunnel's `cfargotunnel.com` name. The Cloudflare connector runs as a separate `cloudflared` service on `rawrarr-roarlay-prod-control`; it exposes no host port. Its root-only connector secret is at `/opt/rawrarr-roarlay-prod/secrets/cloudflared-rawrarr-net-token`. The prior Synkpieces tunnel and token remain untouched.

The durable Compose overlay is [docker-compose.cloudflare-tunnel.yml](../deploy/do3/docker-compose.cloudflare-tunnel.yml). The running host copy is `/opt/rawrarr-roarlay-prod/docker-compose.cloudflare-tunnel.yml`; it uses the cached `linux/amd64` image digest `sha256:faa8e0a1475f137c910be6025f43c8e4092fdd35461b31e19fd1390657a1ff39` with `pull_policy: never`. The service was created under `.container-rebuild-cloudflared.lock` with only the base and tunnel Compose files, `--no-deps --pull never`; the marker was released after verification.

## Cutover and bypass protection

The old direct Rawrarr handler on `rawrarr.willjasen.net` now returns 404. Caddy keeps its public listener and Headscale path handlers; only the final Rawrarr catch-all was changed to `respond "Not Found" 404`. Before the reload, Caddy's source file was copied to `/opt/rawrarr-roarlay-prod/backups/roarlay-prod.Caddyfile.pre-cloudflare-rawrarr-net-20261102`. Restore that file if the legacy route must be reinstated. Do not disable or alter Headscale routes.

## Verification and rollback

- Allowed current client IPv4: `/health` and `/` each returned HTTP 200 through Cloudflare. The current egress address matched the deployed rule.
- From do3's non-allowlisted public IPv4, `https://rawrarr.net/health` returned HTTP 403.
- Direct-origin `https://rawrarr.willjasen.net/` and `/health` both returned HTTP 404; direct-origin `rawrarr.net` returned no HTTP response because Caddy has no site/certificate route for that hostname. Headscale `/version` continued returning HTTP 200.
- Cloudflare reports the tunnel healthy. The connector reports running with zero restarts on the existing control network. Caddy validated and hot-reloaded with zero restarts. The apex DNS records are the proxied tunnel CNAME and the preserved SPF TXT record.
- PostgreSQL, roarlay, Tailscale, and unrelated public services were not recreated. No migration is required because app state, schema, and protocol are unchanged.
- If the tunnel fails, keep the old Rawrarr catch-all closed and repair the connector or disable the new hostname's route while the site is unavailable. Restore the old catch-all only if intentionally ending the IP restriction. Do not delete the existing Synkpieces tunnel or token; do not change Headscale routes or persistent volumes.

## Deployment status

Deployment completed on 2026-11-02. No test suites were run; verification used live DNS, Cloudflare API state, allowed and denied HTTP requests, container health/restart checks, Caddy validation/reload, and the preserved Headscale route.

## References

- [Cloudflare custom rules](https://developers.cloudflare.com/waf/custom-rules/)
- [Allow traffic from IP addresses in an allowlist only](https://developers.cloudflare.com/waf/custom-rules/use-cases/allow-traffic-from-ips-in-allowlist/)
- [Cloudflare Page Rules](https://developers.cloudflare.com/rules/page-rules/)
