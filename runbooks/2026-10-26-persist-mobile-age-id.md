# Keep the age ID after a mobile page refresh

## Objective

Fix issue [#179](https://github.com/willjasen/rawrarr/issues/179): a validated age ID entered in the coordinator's Movies view should be saved before slower node requests begin, so refreshing a mobile page during unlock does not discard it.

## Changes

- Locally validate the entered age ID and persist it before waiting for the metadata node's age-vault request.
- Keep the ID usable in the current page if browser storage is unavailable, and report that it will not survive refresh.
- Handle browsers that throw while reading, writing, or removing browser storage without preventing the workspace from loading.
- Add browser regression coverage that holds the node request open, verifies the ID is already saved, and then reloads.

## Verification

- `node --test scripts/tests/test_rawrarr_coordinator_workspace.mjs` passed: 28 tests.
- The isolated OrbStack runner's `test_rawrarr_coordinator_sonarr_settings.mjs` age-ID test passed in Chromium. It verifies persistence before the delayed node request, then reload restoration and sign-out cleanup.
- The isolated runner's full Node and Rust suites passed. Its overall Playwright suite reported unrelated failures in artwork timing, movie identifiers/links/responsiveness, navigation, shared library, and Sonarr settings tests. Python integration tests were skipped by the runner.
- Direct host execution of the browser test was unavailable because `/usr/local/lib/node_modules/playwright-core/index.mjs` is not installed; the isolated runner supplied the browser dependencies.

## Limitations

If a mobile browser blocks persistent storage, no web page can retain the ID across refresh. The UI now reports that condition rather than implying the ID was saved.
