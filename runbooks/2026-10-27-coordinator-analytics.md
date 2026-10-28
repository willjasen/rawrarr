# Coordinator analytics integration

## Scope and tracking

Implements issue [#255](https://github.com/willjasen/rawrarr/issues/255): optional self-hosted Umami analytics for internal and public coordinator UI deployments.

## Changes

- Added opt-in Umami pageview injection for the coordinator landing/workspace document. Automatic URL and click tracking are disabled; only a fixed `/coordinator` label and title are sent. The coordinator proxy allows only the tracker script and event submission endpoints, using the coordinator's private Docker network.
- Added a Compose overlay for the public and internal do3 coordinator projects.
- Added an Umami Compose overlay with its own PostgreSQL volume, loopback-only dashboard listener, and deployment/privacy documentation.
- Existing Rawrarr databases, synchronized data, credentials, node protocols, and runtime state have no migration or compatibility impact.

## Verification

- Live rollout completed on do3 on 2026-10-28. Internal and public each have a separate Umami 3.4.1 service, PostgreSQL 17 database/volume, website ID, and analytics environment file. The internal site is `rawrarr.risk-mermaid.ts.net`; the public site is `rawrarr.willjasen.net`.
- Both roarlay coordinators run the published Rawrarr revision `d342ca5229b1582df153fe2f33d8c7c0bef45f52` (image `rawrarr:d342ca5229b1-amd64`) and were healthy at verification. Both coordinator health routes returned HTTP 200; both `/analytics/script.js` GET requests succeeded. Public home HTML included its configured site ID.
- Real browser visits to each coordinator produced `/coordinator` pageviews in the corresponding independent Umami database. At the final database audit, internal had 4 pageviews / 2 sessions and public had 2 pageviews / 2 sessions; counts can increase as the sites are used.
- Umami retains analytics indefinitely; there is no purge job. Bootstrap admin passwords were rotated and stored in host-local files with mode 0600. Dashboard listeners remain loopback-only and no Tailscale Serve dashboard handler was added. Enroll dashboard TOTP separately when connecting through an authorized local/SSH-forwarded route.

## Rollback

Clear `RAWARR_ANALYTICS_UPSTREAM` or `RAWARR_ANALYTICS_SITE_ID` and recreate only the affected roarlay service. This disables tracking without changing Umami history. Removing the analytics PostgreSQL volume deletes collected analytics permanently. Runtime verification was performed on do3; no analytics dashboard was exposed for remote browser access.
