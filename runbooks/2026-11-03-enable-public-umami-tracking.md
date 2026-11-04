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

PR [#416](https://github.com/willjasen/rawrarr/pull/416) merged the source changes at `d7b8acfb015894587916a88d55df3473f16143ea`. The public coordinator is healthy on published revision `9407aeb37f12965671c911e0e86e4bac6dfbf940` with its analytics site ID and `http://umami:3000` upstream configured by the persistent host-local override `/opt/rawrarr-roarlay-prod/builds/github-main-d7b8acfb-analytics-settings/analytics-settings.override.yml`. The existing coordinator image and database were preserved.

The Cloudflare home-IP restriction remains in place for all other requests. Its only exceptions are `GET /analytics/script.js` and `POST /analytics/api/send` on `rawrarr.net` and `roarlay.rawrarr.net`; the second host must be allowed because Netlify's same-origin rewrite fetches the tracker through the public coordinator from Netlify egress. This does not expose the coordinator app or the Umami dashboard.

Verification after rollout: both public tracker script routes returned HTTP 200; POSTs for the separate website and coordinator pageview IDs returned HTTP 200, and the public Umami database contained the corresponding fixed `/website` and `/coordinator` pageview rows. `/analytics/login` returned HTTP 404 on both public hosts. No Rawrarr database, synchronized data, node protocol, or existing Umami data migration is required.

## Rollback

Restore the Cloudflare home-IP-only rule, remove the public coordinator analytics settings from its host-local override under the public Roarlay deployment lock, and recreate only public `roarlay` with the existing published image. Revert the website scripts and Netlify rewrites through GitHub if needed. Retain the public Umami database, both new site rows, and its existing site; do not remove or alter the internal Umami service.
