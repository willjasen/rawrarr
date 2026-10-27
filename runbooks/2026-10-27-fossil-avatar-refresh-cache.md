# Reuse generated Fossil avatars after refresh

Avoid rerendering the Fossils page's generated pictures on each browser page refresh (issue [#248](https://github.com/willjasen/rawrarr/issues/248)).

## Findings

The coordinator generated deterministic JXL and PNG image bytes on every cache miss/request reaching the server, invoking the raster and JPEG XL tools even though its stable-seed URLs advertised a one-year private immutable browser cache. Browser caching usually avoided the request, but a cold cache or forced reload still caused unnecessary rendering work.

## Changes

- Cache rendered profile and Fossil avatar bytes in coordinator memory using image role, encoding, renderer generation, and validated seed as the key.
- Bound retained entries to 256; when the bound is reached, clear the cache before adding a new rendering.
- Keep the existing authenticated routes and private immutable response headers. Increment the `v1` renderer key when intentionally changing avatar artwork.
- Add coordinator workspace regression assertions for all four cache keys and the bound.

## Verification

`node --test --test-name-pattern='signed-in profile requests|device pictures use' scripts/tests/test_rawrarr_coordinator_workspace.mjs` passed (2/2). `git diff --check` passed. The full coordinator workspace test file has one unrelated pre-existing failure in the device-name Edit-button assertion (30/31 pass). Rust tests were not run because `cargo` is not installed in this environment.

## Rollback

Remove the in-memory rendered-avatar cache and restore the four handlers to call their renderer directly. No persistent or shared data is written.
