# Movie artwork stability

Issue: https://github.com/willjasen/rawrarr/issues/387

## Cause and change

The coordinator renders cached movie text and starts poster requests while it
fetches the live Radarr library. The live result cleared the grid and created new
image elements, blanking loaded posters and restarting in-flight artwork.

The shared Movies/TV card renderer now reconciles cards by provider library ID,
updates text and availability in place, and retains loaded and pending posters.
Search and filters hide cards instead of rebuilding them. Removed library entries
are removed, new entries are created, explicit artwork URL changes still apply,
and failed artwork can be retried on the next library render. Existing node
changes and private-ID clearing continue to clear the movie grid; restoring the
same selected node no longer clears its already-rendered cached cards.

The saved-cache startup test also exposed an initialization-order failure:
restored movies could call the renderer before the Arr link-state map was
initialized. That map now initializes before any cached rendering.

## Adoption and recovery

This is coordinator browser code only. Existing deployments adopt it through the
normal published coordinator UI update and a browser reload. No data, encrypted
configuration, schema, cache-format, or node protocol changes require migration.
Subscriber services do not need updating. Roll back the coordinator UI bundle to
the previous published revision if needed; there are no persistent-state changes.

## Verification

- Browser regression: delayed live-library completion preserves both loaded and
  in-flight images; unchanged posters are requested once; filtering preserves
  image identity; updated metadata, explicit artwork changes, additions, and
  removals still render.
- Desktop 1280x800 and mobile 375x812 cover long titles, horizontal overflow, and
  the refresh action.
- `python3 scripts/tests/run_rawrarr_tests_in_docker.py`: Node.js (154 tests),
  Playwright (including both new viewport cases), and Rust (121 tests) passed in
  `rawrarr-test-regression-tests-mw0f7lxa`. Initial test runs exposed the cached
  startup and same-node clearing problems above; both were fixed before this
  passing run.
- Disposable Python integration and multi-node environment suites were not run;
  this change does not touch their backend or protocol behavior.
- Live internal deployment is not part of this change's verification.
