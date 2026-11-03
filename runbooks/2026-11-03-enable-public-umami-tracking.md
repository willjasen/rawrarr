# Enable public Umami tracking

Issue: [#412](https://github.com/willjasen/rawrarr/issues/412)

## Finding

On 2026-11-03, `https://roarlay.rawrarr.net/analytics` returned HTTP 401 and the coordinator's `/analytics/script.js` and `/analytics/api/send` returned 404. The public Umami service itself was healthy on do3 at loopback port 7301, but the public coordinator had no `RAWARR_ANALYTICS_*` settings. Its dashboard is intentionally not exposed publicly. The internal Umami instance has a separate database and must not be used by the public services.

The public Umami admin password was available in its existing root-only host file, so no password reset was needed. Added two sites to the public Umami database through its authenticated API while keeping the password and token out of command output:

- `Rawrarr Coordinator`, domain `roarlay.rawrarr.net`, website ID `1a81b3b2-5480-46e6-923f-044734a5d545`.
- `Rawrarr Website`, domain `rawrarr.net`, website ID `72da7a2e-fe1b-4058-b07b-912d526c939e`.

The existing `Rawrarr Public` site remains unchanged.

## Implementation

- Configure the public coordinator with its dedicated Umami website ID and `http://umami:3000` upstream. The coordinator proxy allows only `GET /analytics/script.js` and `POST /analytics/api/send`; it does not serve the Umami dashboard.
- The static `rawrarr.net` page loads its separate website ID with automatic URL collection disabled and sends one fixed `/website` page label.
- Netlify rewrites only `/analytics/script.js` and `/analytics/api/send` to the public coordinator, keeping browser requests same-origin. The site Content Security Policy remains same-origin-only.
- Keep the dashboard on the internal tailnet route. Production and internal Umami databases, IDs, secrets, and networks remain separate.

No Rawrarr database, synchronized data, node protocol, or existing Umami data migration is required. New Umami website rows are additive and can remain after rollback without affecting existing sites.

## Rollout and verification

The public coordinator's published revision `9407aeb37f12965671c911e0e86e4bac6dfbf940` contains the opt-in analytics proxy. During the attempted configuration rollout, the live image changed multiple times between `9407aeb37f12965671c911e0e86e4bac6dfbf940` and `e10250a8ef8ff694b41c44609c2a4962d7d3c030`, indicating concurrent deployments. The configuration rollout was stopped rather than replacing a changing image. At the final check, public roarlay was healthy on revision `9407aeb37f12965671c911e0e86e4bac6dfbf940`, the deployment lock was absent, and no analytics environment settings were present. `https://roarlay.rawrarr.net/analytics/script.js` returned HTTP 404 and `/health` returned HTTP 200. A host-local override was created under `/opt/rawrarr-roarlay-prod/builds/analytics-enable-20261103-r1/` but was not left active in the public service definition.

The site entries are present in the public Umami database, but the coordinator and static site changes are not deployed. A check of the public tracker URL through Cloudflare returned HTTP 403 during the attempt, so the ingress behavior also needs verification before enabling public website rewrites.

After the source change is published and the public deployment is stable, apply the coordinator's analytics settings using a host-local Compose override under the public Roarlay deployment lock; recreate only public `roarlay` with `--no-deps --pull never`, preserving the image, Umami service/database, PostgreSQL, Caddy, Cloudflare Tunnel, and internal services. Verify local and public tracker routes, coordinator health, and that `/analytics/login` remains unavailable on `roarlay.rawrarr.net`. Once Netlify's normal GitHub deployment publishes the website scripts and endpoint rewrites, verify the same-origin tracker loads, the fixed `/website` event is submitted, and the event appears in the public Umami dashboard. Confirm `rawrarr.net/analytics/login` is not proxied.

## Rollback

Under the public Roarlay deployment lock, remove the public coordinator analytics settings from its host-local override and recreate only public `roarlay` with the existing published image. Revert the website scripts and Netlify rewrites through GitHub if needed. Retain the public Umami database, both new site rows, and its existing site; do not remove or alter the internal Umami service.
