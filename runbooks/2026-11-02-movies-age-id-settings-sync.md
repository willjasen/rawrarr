# Movies age ID settings synchronization

Issue: https://github.com/willjasen/rawrarr/issues/338

## Change

The coordinator workspace now listens for same-origin `localStorage` changes to the browser age ID. When Settings imports an ID in another tab, an already open Movies tab adopts it and runs its existing unlock flow. When another tab clears the ID, Movies clears the rendered library and reports that metadata is locked.

The ID remains browser-local. This change does not alter synchronized files, database schemas, node protocols, or persisted application data, so existing deployments need no migration or compatibility rollout.

## Verification

- Added `scripts/tests/test_rawrarr_age_id_cross_tab_browser.mjs` for the cross-tab import and responsive layout flow.
- The local browser test could not run because `/usr/local/lib/node_modules/playwright-core/index.mjs` is absent in this checkout. Run the test in the isolated Playwright test image as documented in `scripts/tests/README.md`.
