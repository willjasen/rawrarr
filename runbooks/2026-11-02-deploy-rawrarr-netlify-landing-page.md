# Deploy the rawrarr.net landing page

Issue: [#293](https://github.com/willjasen/rawrarr/issues/293)

## Scope

Deploy the static page from `website/` to `rawrarr.net` on Netlify. The main Rawrarr app is `https://roarlay.rawrarr.net/`; both the page header and hero action link there. The page remains independent from the coordinator service and does not publish app or subscriber runtime data.

## Current state

- Read-only HTTPS to `https://roarlay.rawrarr.net/` returned HTTP 200.
- The `rawrarr.net` zone delegates to Cloudflare nameservers. Public DNS returned no A, AAAA, or CNAME records for the apex when checked on 2026-11-02.
- Will reports that the Netlify project has been set up. No Netlify CLI session, Netlify integration, site ID, or auth token is available in this workspace, so that project could not be inspected; token values were not read.
- No production deployment or DNS record has been created or changed by this task.

## Deployment and DNS steps

1. Confirm the Netlify project is connected to the private `willjasen/rawrarr` GitHub repository with `main` as the production branch. If not, connect it. The tracked `netlify.toml` publishes only `website/` and sets baseline security headers.
2. Add `rawrarr.net` as the site’s custom domain in Netlify and retrieve the exact apex DNS target(s) Netlify provides.
3. In Cloudflare, add only those Netlify-provided apex records; retain the existing Cloudflare nameservers and do not change records for `roarlay.rawrarr.net`.
4. Wait for Netlify domain validation and HTTPS provisioning. Verify the Netlify production URL and `https://rawrarr.net/`, confirm the security headers, and confirm the app links to `https://roarlay.rawrarr.net/`.

## Compatibility and rollback

There is no existing Netlify site state or application data migration. This change has no persistent data, schema, encrypted configuration, node protocol, or coordinator deployment impact. A DNS change can be rolled back by restoring the prior apex records (none were present at the time of this runbook); a Netlify deployment can be rolled back to its prior published version. Preserve records for the app subdomain during either rollback.

## Verification

`netlify.toml` parsed successfully. In the full isolated regression run, all 146 Node tests and all 115 Rust tests passed; the landing-page Playwright check passed at 1280×800 and 375×812, including its app URL assertions. The broader Playwright suite had one unrelated timeout in `test_rawrarr_coordinator_sonarr_settings.mjs` while waiting for the Radarr setup note. The runner therefore exited with a failing Playwright suite.

Before the DNS cutover, verify a successful Netlify production deploy, domain validation, HTTPS, and the response headers at `rawrarr.net`.
