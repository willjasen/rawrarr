# Redirect Roarlay root visits to authentication

Issue: [#333](https://github.com/willjasen/rawrarr/issues/333)

## Scope

For signed-out visitors, the coordinator root (`/`) now redirects to `/auth`. Authenticated visitors still receive the workspace at `/`, and signed-in visits to `/auth` continue to redirect back to `/`. The separate Netlify landing page remains at `rawrarr.net`.

## Compatibility and adoption

This changes only the HTTP response for signed-out `GET /`; there is no persistent data, schema, secret, node protocol, or synchronized-file change and no migration is needed. Existing coordinator deployments adopt the behavior when updated with the published application image from GitHub `main`. Rollback is to the prior published application revision; no state rollback is required.

## Verification

The focused Chromium regression passed in the isolated OrbStack test image: the root returned HTTP 303 with `Location: /auth`, and the sign-in page rendered. All 146 Node tests and all 115 Rust tests passed. The wider Playwright suite had two unrelated failures: a timeout waiting for the Radarr setup note in `test_rawrarr_coordinator_sonarr_settings.mjs` and a desktop section-title visibility assertion in `test_rawrarr_signing_topology_browser.mjs`. Browser verification completed with those failures; Python integration tests were skipped by the runner because they require disposable service fixtures.
