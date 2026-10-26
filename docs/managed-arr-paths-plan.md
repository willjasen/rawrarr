# Managed Radarr and Sonarr paths

**Status:** Planned; this document records the design boundary and implementation sequence. No proxy route is implemented yet.
**Date:** 2026-10-26
**Tracking:** [Issue #116](https://github.com/willjasen/rawrarr/issues/116); managed app provisioning is separately tracked in [issue #96](https://github.com/willjasen/rawrarr/issues/96).

## Goal

Let an owner open a Radarr or Sonarr UI that Rawrarr provisioned inside the same subscriber Docker environment through a path on the Rawrarr origin. The external Arr URL and API key that a user may configure for metadata access do not identify or enable a managed UI route.

## Current state

- The subscriber app has connection settings for a Radarr and a Sonarr endpoint. Those values are used for API access and may point outside Rawrarr.
- Coordinator requests to a subscriber currently use authenticated paths under `/nodes/{node_id}/...`; the coordinator restricts which subscriber API paths it relays.
- Subscriber Compose generation does not currently install Radarr or Sonarr. The isolated OrbStack test Compose includes internal Arr containers for connectivity checks.
- Issue #96 tracks creating a managed Arr instance from a user's existing installation. Provisioning needs to make the app reachable on the subscriber's private Docker network before this path feature can be used.

## Proposed behavior

- Provide stable, node-scoped paths such as `/nodes/{node_id}/apps/radarr/` and `/nodes/{node_id}/apps/sonarr/` on the authenticated Rawrarr coordinator origin.
- Resolve each path only to its corresponding managed service on the selected subscriber's private Docker network. Do not publish Arr ports to the host for this route.
- Keep the managed service's own UI authentication enabled. Continue to require the Rawrarr account session and an owner-authorized subscriber route.
- Render an Open Radarr or Open Sonarr link only when that managed service is provisioned and reachable. Explain when a service is absent or offline.
- Keep managed-service discovery/configuration separate from the external Radarr/Sonarr metadata connection settings. Never use an external URL, API key, or UI credentials as a proxy target or path toggle.

## Design constraints and open implementation work

1. Coordinate the Compose contract with issue #96: use fixed service identities and private Docker DNS names, and configure each Arr app's URL base for its Rawrarr path as part of provisioning. Do not accept an arbitrary browser-supplied upstream URL.
2. Extend the subscriber and coordinator routing with a narrow allowlist for only the two managed app prefixes. Keep normal Rawrarr API relay restrictions unchanged. Require authenticated owner access and an approved metadata node.
3. Preserve Arr UI cookies and authentication while scoping cookies to the node/app path. Rewrite redirects and ensure scripts, styles, images, API calls, and URL-base handling remain under that prefix. Reject redirects to unrelated hosts.
4. Bound request and response handling, preserve streaming where possible, and avoid logging query strings, cookies, authorization headers, or app request bodies.
5. Add UI status and links based on managed service health, never based on whether an external Arr endpoint is configured.
6. Add regression coverage for both managed apps, path-prefix behavior, authentication, unavailable containers, and proof that external connection settings cannot alter the upstream.

## Acceptance

- Each provisioned internal app opens from its stable Rawrarr path and stays on that path through redirects and asset/API requests.
- Requests can reach the app through the private Compose network without a host-published Arr port.
- The route is unavailable for external-only connections, unapproved nodes, missing services, and offline services.
- Radarr/Sonarr UI authentication remains active; secrets do not appear in the route URL, logs, or coordinator configuration.
- Focused Rust/browser and isolated Compose checks cover both services and the negative cases before the feature is considered complete.
