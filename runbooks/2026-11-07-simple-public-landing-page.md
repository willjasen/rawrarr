# Simple public landing page

Issue: [#443](https://github.com/willjasen/rawrarr/issues/443)

## Scope and design

Replace the multi-section `rawrarr.net` introduction with a single narrow column: logo and tagline, a short product explanation, a short device/coordinator explanation, and one **Open app** link. Keep the current app destination, static assets, same-origin analytics, system light/dark palettes, visible early-development caveat, and expandable privacy/analytics disclosure. Authenticated pages and coordinator sign-in routes are unchanged.

## Adoption and rollback

The existing Netlify deployment publishes `website/` from GitHub `main`. Adoption requires publishing this task's reviewed changes to `main`; no coordinator or subscriber container rebuild is needed. No persistent data, schema, encrypted format, or protocol changes occur, so no migration is required. Roll back by reverting the website changes through a focused PR and letting Netlify publish that `main` revision.

## Verification

Passed `node --test scripts/tests/test_public_landing_page.mjs` in a uniquely named, network-disconnected clone of the existing `rawrarr-testing-base:local` OrbStack image, with the current website, Netlify configuration, and focused test copied into it. The clone was removed afterward.

Coverage includes the single app link and destination, desktop 1280x800 and mobile 375x812 layout, light/dark palettes, keyboard navigation, disclosure expansion, longer wrapping content, and unchanged privacy-limited analytics. The app destination is intercepted with a synthetic response to verify navigation without contacting the live application. `git diff --check` also passed. Full application suites were not run; authenticated app files were not changed. The live Netlify page has not been deployed or verified by this task.
