# Coordinator analytics integration

## Scope and tracking

Implements issue [#255](https://github.com/willjasen/rawrarr/issues/255): optional self-hosted Umami analytics for internal and public coordinator UI deployments.

## Changes

- Added opt-in Umami pageview injection for the coordinator landing/workspace document. Automatic URL and click tracking are disabled; only a fixed `/coordinator` label and title are sent. The coordinator proxy allows only the tracker script and event submission endpoints, using the coordinator's private Docker network.
- Added a Compose overlay for the public and internal do3 coordinator projects.
- Added an Umami Compose overlay with its own PostgreSQL volume, loopback-only dashboard listener, and deployment/privacy documentation.
- Existing Rawrarr databases, synchronized data, credentials, node protocols, and runtime state have no migration or compatibility impact.

## Verification

- Pending live rollout. Each Umami service must be deployed, configured with an administrator and 2FA, assigned a retention period, and given an environment-specific website before enabling tracking.

## Rollback

Clear `RAWARR_ANALYTICS_UPSTREAM` or `RAWARR_ANALYTICS_SITE_ID` and recreate only the affected roarlay service. This disables tracking without changing Umami history. Removing the analytics PostgreSQL volume deletes collected analytics permanently.
