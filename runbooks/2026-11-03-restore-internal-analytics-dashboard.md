# Restore the internal analytics dashboard route

Issue: [#279](https://github.com/willjasen/rawrarr/issues/279)

## Finding

On 2026-11-03, the internal Tailscale Serve configuration had only the HTTPS `/` handler, pointing directly to `http://127.0.0.1:8080`. This preserved coordinator access but bypassed the already-running internal Caddy dashboard proxy on port 8081. The coordinator health page returned HTTP 200, while `/analytics` and `/analytics/script.js` returned HTTP 401.

The internal Caddy and Umami containers were running. A local request to Caddy at `/analytics` returned the Umami page, and the internal Umami database reported no pending migrations.

## Repair

Preserve the root handler to roarlay and add a path-specific internal Serve handler:

- `/` -> `http://127.0.0.1:8080`
- `/analytics` -> `http://127.0.0.1:8081/analytics`

The upstream `/analytics` path is required because the Serve mount removes the public mount prefix before proxying. Pointing the mount at `http://127.0.0.1:8081` instead sent requests to Caddy's fallback route and did not reach Umami.

No containers were recreated. No application code, credentials, database, data volume, or public coordinator route was changed. No migration is required.

## Verification

- Tailscale Serve status reports both handlers above.
- Internal `/analytics` and `/analytics/login` return HTTP 200; the dashboard page title is `Umami`.
- Internal `/analytics/script.js` returns HTTP 200 with a JavaScript content type.
- Internal `/health` returns HTTP 200 and identifies roarlay.
- The public coordinator's `/analytics` remains HTTP 404.
- The Tailscale deployment lock was absent after verification.

Umami authentication remains required to use the dashboard.

## Rollback

Under the internal Tailscale deployment lock, remove only the dashboard mount from the internal Tailscale sidecar:

```sh
tailscale serve --https=443 --set-path=/analytics off
```

Confirm that the root `/` handler still points to `http://127.0.0.1:8080`, and verify internal HTTPS `/health`. Do not reset the full Serve configuration.
