# Playwright coverage for responsive Devices layout

Add rendered-browser checks for the mobile layout work tracked in [issue #193](https://github.com/willjasen/rawrarr/issues/193), and make responsive browser checks a standing requirement for future UI changes.

## Change

Extended `test_rawrarr_topology_visibility_browser.mjs` to load the actual coordinator workspace with long device names at desktop (1280×800) and narrow mobile (375×812) viewports. The test checks horizontal overflow, heading and Add action bounds, device card and status bounds, topology node bounds, desktop multi-column layout, and that Edit and Delete actions remain within the viewport.

Updated `AGENTS.md` to require Playwright coverage at both viewports for browser-facing UI changes, including overflow and control reachability checks. Added the new coverage to the test index.

This task changes test and contributor guidance only; no runtime migration or deployment is needed.

## Verification

- `test_rawrarr_topology_visibility_browser.mjs` — all 6 tests passed in the isolated OrbStack test image; the responsive case also passed alone after its final viewport-bound assertions were added.
- `git diff --check` — passed.
