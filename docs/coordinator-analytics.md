# Coordinator analytics

Rawrarr can optionally send one anonymous page-view event from the coordinator landing/workspace page to a self-hosted Umami instance. Analytics is off unless both `RAWARR_ANALYTICS_UPSTREAM` and `RAWARR_ANALYTICS_SITE_ID` are configured. It never tracks subscriber pages, APIs, health checks, node relays, or user actions.

The integration disables Umami's automatic tracking and submits only a fixed `/coordinator` page label and title. It does not send the browser URL, query string, referrer, account/device/node identifiers, library data, or event properties. The site ID must contain only letters, digits, hyphens, or underscores. The server proxy only allows `http://umami:3000` and the Umami tracker and event routes. Invalid or incomplete settings leave tracking disabled. The coordinator forwards the client IP and browser user-agent to Umami so it can calculate visitor metrics; Umami uses these to derive anonymous session metrics. Self-hosting controls where this data is stored but does not mean no visitor data is processed. Review Umami's [metric definitions](https://docs.umami.is/docs/metric-definitions).

## Deployment

The normal coordinator deployment does not include Umami or its PostgreSQL database. The coordinator defaults both analytics settings to empty, so adding `docker-compose.analytics.yml` leaves tracking disabled and starts no analytics services.

To opt in an environment, first create its protected Umami credentials and private Docker network, then explicitly start the analytics services with `docker-compose.analytics-services.yml`. Set `RAWARR_ANALYTICS_NETWORK` to that coordinator project's private Docker network (the current do3 networks are `rawrarr-internal-do3-control` and `rawrarr-roarlay-prod-control`). Umami joins that private network, while its PostgreSQL port is not published. The Umami web service binds only to host loopback port 7300 for administration. For example:

```sh
docker compose -p rawrarr-internal-do3 --env-file /path/to/protected/analytics.env \
  -f docker-compose.analytics-services.yml up -d
```

Use the matching coordinator Compose project name for each environment so its Umami database volume stays separate.

For coordinator tracking, set `RAWARR_ANALYTICS_UPSTREAM=http://umami:3000` and `RAWARR_ANALYTICS_SITE_ID` to the environment's Umami website ID in its protected deployment environment. Then recreate only `roarlay` with the coordinator analytics overlay. Use a separate Umami website/site ID and database volume for each environment. The analytics database is not Rawrarr PostgreSQL and contains no Rawrarr application records.

The public coordinator exposes only tracker and event endpoints at its existing HTTPS origin under `/analytics`; its dashboard is not exposed through the public origin. The internal do3 coordinator mounts `/analytics` through Tailscale Serve to `http://127.0.0.1:8081/analytics`, where the internal Caddy sidecar proxies to Umami. The coordinator root route remains a direct Serve proxy to `http://127.0.0.1:8080`. Include `/analytics` in the upstream URL because the Serve mount strips that prefix before forwarding the request. The custom Umami image configured by `deploy/do3/docker-compose.analytics-dashboard-internal.yml` must be built with `BASE_PATH=/analytics` before that dashboard image can be deployed. The normal coordinator deployment neither builds nor starts this image. Keep the existing internal Umami container and data in place when deploying coordinator changes.

The public `rawrarr.net` landing page uses its own website ID in the public Umami database. Netlify rewrites only `/analytics/script.js` and `/analytics/api/send` to the corresponding public coordinator endpoints, so browser requests remain same-origin and the site Content Security Policy can continue to allow only `'self'`. The public coordinator proxy accepts only the tracker script and event POST; it does not expose Umami dashboard routes. Do not point the public site at the internal Umami instance: production and internal databases, site IDs, and credentials remain isolated.

Do not expose PostgreSQL. Create a strong independent DB password, an application secret, and a 64-character hex 2FA encryption key in each host's protected environment file. Keep that file out of Git and `/shared`. Self-hosted Umami retains analytics records indefinitely; no expiration job is configured. Back up the Umami database separately and test restore before upgrades. The Compose image is pinned to Umami `3.4.1`; review upstream security releases before changing the tag. Upgrade the analytics service independently from Rawrarr.

Verify the browser loads `/analytics/script.js`, posts only the fixed `/coordinator` pageview to `/analytics/api/send`, and the event appears in that environment's dashboard. On `rawrarr.net`, verify the Netlify same-origin rewrites return the tracker and accept the fixed `/website` event; the Umami dashboard and all other API routes must remain unavailable publicly. To disable coordinator tracking, clear either coordinator setting and recreate roarlay. To disable website tracking, remove its tracker scripts. Umami and its volume can remain running. Deleting the Umami volume permanently deletes analytics history.

This adds a separate analytics data domain but changes no Rawrarr persistent data, schema, node protocol, or synchronized `/shared` content. No migration is required. Existing Umami containers and their data are not removed or changed by this code update; services should only be started through the explicit analytics services Compose file.
