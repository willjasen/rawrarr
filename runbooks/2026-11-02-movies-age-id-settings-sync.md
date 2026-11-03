# Movies age ID settings synchronization

Issue: https://github.com/willjasen/rawrarr/issues/338

## Change

The coordinator workspace now listens for same-origin `localStorage` changes to the browser age ID. When Settings imports an ID in another tab, an already open Movies tab adopts it and runs its existing unlock flow. When another tab clears the ID, Movies clears the rendered library and reports that metadata is locked. Movies also updates its initial status when an ID is already available, rather than continuing to prompt for an ID that the page has loaded.

The ID remains browser-local. This change does not alter synchronized files, database schemas, node protocols, or persisted application data, so existing deployments need no migration or compatibility rollout.

## Verification

- `node --test scripts/tests/test_rawrarr_coordinator_workspace.mjs` passed (34 tests); JavaScript syntax and `git diff --check` passed.
- `test_rawrarr_age_id_cross_tab_browser.mjs` passed in the isolated OrbStack runner, including the no-node case where Movies must not ask for an ID already saved in this browser.
- The full isolated runner reported unrelated existing failures: `test_netlify_ignore_build.mjs` could not find the root `netlify-ignore-build.sh` fixture in the test container, and the signing-topology browser test failed its desktop-title visibility assertion. All other reported Node and browser checks passed.
- The initial cross-tab update is deployed on do3 at revision `7af3d278`. The follow-up status-message correction is pending its own merge and deployment.
