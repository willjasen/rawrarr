# Movies age ID settings synchronization

Issue: https://github.com/willjasen/rawrarr/issues/338

## Change

The coordinator workspace now listens for same-origin `localStorage` changes to the browser age ID. When Settings imports an ID in another tab, an already open Movies tab adopts it and runs its existing unlock flow. When another tab clears the ID, Movies clears the rendered library and reports that metadata is locked. Movies also updates its initial status when an ID is already available, rather than continuing to prompt for an ID that the page has loaded.

The ID remains browser-local. This change does not alter synchronized files, database schemas, node protocols, or persisted application data, so existing deployments need no migration or compatibility rollout.

## Verification

- `node --test scripts/tests/test_rawrarr_coordinator_workspace.mjs` passed (34 tests); JavaScript syntax and `git diff --check` passed.
- `test_rawrarr_age_id_cross_tab_browser.mjs` passed in the isolated OrbStack runner, including the no-node case where Movies must not ask for an ID already saved in this browser.
- The full isolated runner reported unrelated existing failures: `test_netlify_ignore_build.mjs` could not find the root `netlify-ignore-build.sh` fixture in the test container, and the signing-topology browser test failed its desktop-title visibility assertion. All other reported Node and browser checks passed.
- PR [#339](https://github.com/willjasen/rawrarr/pull/339) deployed to do3 from `7af3d278`; follow-up PR [#346](https://github.com/willjasen/rawrarr/pull/346) deployed from current `main` at `b29e11b076c7102e2865706a8f1a8bf778e6282b`.
- On do3, fetched the current-main archive directly from GitHub using the protected read token. Archive SHA-256: `bf29273f4105a36303d0ed83481335d42d3f135e57796ea5c3ab59313620ecb5`. Built native `linux/amd64` image `sha256:6abd868bb1a70b405d1462bd9e7684cb10ae9572d0c70d46ee4a69977b15b287`; retained the prior base and running service image as `rawrarr:rollback-base-7af3d27-amd64` and `rawrarr:rollback-service-7af3d27-amd64`.
- Recreated only `rawrarr-internal-do3-roarlay` with `--no-deps --pull never`. The service is healthy; local and HTTPS `/health` pass; Tailscale Serve still maps `/` to `http://127.0.0.1:8080`; the served workspace asset contains the saved-ID status message and cross-tab handler. Released the roarlay deployment lock after verification.
