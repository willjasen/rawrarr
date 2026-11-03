# Coordinator favicon

Issue: https://github.com/willjasen/rawrarr/issues/275

## Change

Coordinator UI pages now advertise the existing Rawrarr SVG favicon, including sign-in, age setup, the main workspace, health, passkey, and billing pages. Roarlay serves the same asset from `/favicon.ico` as well as `/assets/rawrarr-favicon.svg`, covering clients that request the conventional path.

## Adoption and compatibility

This is a static UI and HTTP route change. It does not change persistent data, database schemas, encrypted configuration, or node/coordinator protocols, so existing state requires no migration. Deploy the published application image to the internal coordinator service on do3 to adopt the change; subscriber nodes do not need an update. Public nodes adopt it when they update to a release containing the change.

The icon remains in the existing UI bundle and has the existing embedded fallback, so both published UI bundles and first-boot fallback assets can serve it.

## Verification

No automated tests or environment deployment were run for this change. Source inspection confirmed the favicon route uses the same SVG handler and UI bundle path as the existing asset route, and coordinator HTML entry pages reference that asset.
