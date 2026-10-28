# rawrarr.net Netlify landing page

Issue: [#293](https://github.com/willjasen/rawrarr/issues/293)

## Scope

Add a public, static introduction to Rawrarr at `rawrarr.net`, separate from the authenticated coordinator and its signed-out landing page. Netlify publishes only `website/`; it does not build or expose the Rust service. The content describes the current Radarr/Sonarr, device, storage, and privacy boundaries and identifies the project as early, actively changing, and not independently audited.

The page is self-contained and uses the existing Rawrarr logo. It has no application API calls, analytics, external assets, cookies, account forms, secrets, or subscriber data. A restrictive Content Security Policy and baseline response headers are defined in `netlify.toml`.

## Adoption and compatibility

Existing deployments do not need a migration. This is a new static site with no persistent state, schema, encrypted configuration, or node protocol changes. It does not alter the internal coordinator or subscriber services.

To adopt the page, connect the repository to a Netlify site and set the production branch to `main`; leave the build command blank and set the publish directory to `website` (both settings are captured by `netlify.toml`). Configure `rawrarr.net` as the primary custom domain in Netlify and follow the DNS records Netlify provides for that site. Enable HTTPS and redirect any chosen `www` hostname to `rawrarr.net`. Verify the production deployment at both the Netlify URL and `https://rawrarr.net/`, and check the response security headers. Keep public publication of the repository as a separate decision.

## Verification

The focused Playwright check is `node --test scripts/tests/test_public_landing_page.mjs` in the Rawrarr Playwright test image. It passed in a uniquely named offline OrbStack clone. It verifies the static page and Netlify publish path, meaningful page content, rendered desktop and mobile layouts at 1280×800 and 375×812, horizontal overflow, primary action geometry, and local-only asset loading. `netlify.toml` also parsed successfully with Python's TOML parser.

The full regression runner reported 144 Node tests and 113 Rust tests passing. Its broader Playwright suite had an unrelated failure in `test_rawrarr_crypto_ui.mjs`: the billing test expects the old “Subscription and ETH donations” text in the coordinator workspace after the billing controls moved to the Subscription page. The landing-page Playwright check passed when rerun by itself after adding its copied logo fixture.

The Netlify site, custom domain, DNS, and production HTTPS route have not been configured or verified. The Netlify CLI is not installed and no Netlify connection is available in this workspace. Verify them after connecting the repository to a Netlify site. Rollback is to restore the previous Netlify deployment or revert the site commit; no application data rollback is needed.
