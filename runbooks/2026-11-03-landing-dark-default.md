# Dark default for the public Rawrarr landing page

Issue: [#332](https://github.com/willjasen/rawrarr/issues/332)

## Scope and compatibility

The static Netlify page at `https://rawrarr.net/` uses a dark palette by default when the browser reports no color preference. CSS still follows explicit system light and dark preferences. The page remains static and script-free; this change does not affect persistent data, schemas, or application protocols, so no migration is needed.

## Adoption and rollback

Netlify publishes the tracked `website/` directory from `main`; its build-ignore configuration continues to limit builds to website or Netlify configuration changes. Merge this update to adopt it. To roll back, revert the landing page theme change and publish the resulting `main` revision.

## Verification

`scripts/tests/test_public_landing_page.mjs` checks the default no-preference palette, both explicit system palettes, and usability at 1280×800 and 375×812 with no horizontal overflow. Result: `test_public_landing_page.mjs` passed, including rendered explicit light/dark palettes, desktop and mobile layout, and the dark fallback declaration. The complete isolated regression run remains red for unrelated existing failures: missing root `netlify-ignore-build.sh` test fixture, coordinator Sonarr settings timeout, and signing topology title visibility. Rust unit tests passed (116). Python integration tests were skipped by the runner. The dark fallback is asserted from CSS because Chromium `no-preference` resolves to light in this test environment; explicit rendered dark and light modes are both exercised.
