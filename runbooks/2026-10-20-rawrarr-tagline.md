# Add the Rawrarr tagline

## Objective

Show “all the arrs” beneath the rawrarr name beside the logo at the top left of the public landing page.

## Changes

- Add a compact second line to the landing-page header brand lockup, styled to fit desktop and mobile layouts.
- Record the tagline placement in [product and interface design](../docs/DESIGN.md).
- Extend the Chromium landing-page regression test and test index to cover the tagline.

## Verification and deployment

- The Chromium landing-page test passed in `rawrarr-testing-base:local`, including a check that the tagline appears below the product name.
- The migration inventory test and `git diff --check` passed.
- Built the coordinator image from the deployed source snapshot with the tagline and missing SQLx migration 6, then recreated only `roarlay` in the existing `rawrarr-local` Compose project. PostgreSQL and metadata were not recreated.
- Confirmed roarlay is healthy with zero restarts and no `VersionMissing(6)` in its logs. PostgreSQL still reports successful SQLx migrations 1 through 6.
- Verified the tagline and health page at `http://127.0.0.1:7297/` and `https://rawrarr.risk-mermaid.ts.net/`. Tailscale Serve still routes HTTPS `/` to roarlay at `http://127.0.0.1:8080`, and `/direct-api` remains configured.
- Acquired and released `.container-rebuild-roarlay.lock`.

## Final state

- The public landing-page header shows “all the arrs” below the rawrarr name beside the logo. The corrected coordinator image is running on roarlay without changing PostgreSQL migration history.
