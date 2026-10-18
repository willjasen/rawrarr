# Rename the Series menu to TV

## Objective

Use “TV” for the Sonarr section in the workspace navigation and page heading.

## Actions

- Changed the left navigation label and active page heading to “TV”.
- Kept the internal `series` view key and Sonarr API terminology unchanged.
- Extended the navigation test to cover the TV label, selected view, and refresh persistence.

## Verification

- `test_rawrarr_navigation.mjs` passed in the isolated Playwright test container. The local invocation could not resolve its configured Playwright module path.
- `git diff --check` and `node --check scripts/tests/test_rawrarr_navigation.mjs` passed.
- The broader container suite had unrelated existing failures (age setup, artwork timing, direct-route, Node isolation, movie links, shared-library formatting) and the Rust tests could not resolve missing offline crates (`p256`, `aes-gcm`).

## Final state

- The Sonarr section is presented as “TV”.
