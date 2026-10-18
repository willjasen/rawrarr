# Fix direct Sonarr library 404

## Objective

Allow the TV page to load the configured Sonarr library through the metadata node's direct API listener.

## Initial state

The TV workspace requests `/api/sonarr/series` over the selected metadata node's direct route. The normal metadata API had the Sonarr routes, but its dedicated direct API router exposed the settings endpoints without the Sonarr status or series read endpoints. Chrome showed HTTP 404 in the Sonarr library panel.

## Actions

- Added `/api/sonarr/status` and `/api/sonarr/series` to the direct protected metadata router, preserving its existing session middleware.
- Added a source regression assertion that the direct router includes both handlers.

## Verification

- `node --test scripts/tests/test_rawrarr_coordinator_workspace.mjs` passed (13 tests).
- `node --test scripts/tests/test_rawrarr_coordinator_sonarr_settings.mjs` passed in the isolated test image (4 tests).
- `cargo test ... sonarr::` passed in the isolated test image (3 tests).
- Local `cargo test --locked --offline --manifest-path rust-arr/Cargo.toml sonarr::` could not start because the Homebrew `rustc`/LLVM installation is missing `libz3.5.1.dylib`.
- The full container suite was attempted. It reported failures outside this change: 13 Node tests and 3 Rust tests; some existing tests also require local fixtures or installed image tools. The relevant focused tests passed.
- Rebuilt only the `metadata` service in the existing `rawrarr-local` Compose project. It became healthy; unauthenticated direct series access returned HTTP 401, confirming the protected route exists. The existing Tailscale Serve mapping remains `/` to `http://127.0.0.1:8080` and `/direct-api` to `http://127.0.0.1:8082`; the public roarlay health check returned HTTP 200.
- Verified in the authenticated Chrome session at `#tv`: the Sonarr library loaded successfully and displayed its library rows and summary statistics without the HTTP 404.
- `git diff --check` passed for the task-owned files.

## Recovery

Remove the two Sonarr route registrations from `direct_protected` and the regression assertion. The general metadata API routes remain unchanged.
