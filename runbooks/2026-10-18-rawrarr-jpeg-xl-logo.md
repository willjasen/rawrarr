# Serve the rawrarr logo as JPEG XL

## Objective and initial state

Prepare the existing static rawrarr logo for future JPEG XL animation while retaining broad browser compatibility. The original asset was a 2172×724 RGBA PNG used by the setup and workspace pages; the Rust server exposed only the PNG route. The source-control tree already had unrelated user changes in `docs/node-types.md`, `runbooks/README.md`, and the untracked container-distribution runbook; these remain outside this task.

## Changes

- Encoded `rust-arr/src/assets/rawrarr-logo.jxl` from the PNG with `cjxl --distance=0 --effort=9` (lossless JPEG XL).
- Added an `image/jxl` asset route and made both pages prefer JXL via `<picture>`, retaining the existing PNG as the fallback.
- Updated the setup-page regression test and test index, product-design guide, and technical-status document. The logo is still static; occasional animation with reduced-motion handling remains planned.

## Verification

- `cjxl` encoded the 647 KiB PNG to a 337 KiB JXL file. `djxl` decoded it, and FFmpeg raw RGBA output hashes matched exactly between source and decoded images.
- `node --test scripts/tests/test_rawrarr_age_setup.mjs scripts/tests/test_rawrarr_brand_case.mjs` passed (2 tests).
- Docker rebuilt `rawrarr:local`; the local `rawrarr` service was recreated with `--no-deps`, leaving PostgreSQL running on its existing volume.
- Live requests returned HTTP 200 with `image/jxl` and `image/png` content types. Downloaded bytes matched both source assets exactly, `/health` returned `ok`, and both `rawrarr-local` containers report healthy.
- `git diff --check` passed. Native `rustc`/`rustfmt` could not start because the workstation's Homebrew LLVM expects a missing `libz3.5.1.dylib`; the Docker release build compiled successfully instead.

## Final state

The JXL asset is ready for supported browsers, with the PNG fallback retained. Animation has not been created. Browser support varies: Safari supports JPEG XL, while current Chrome support is still behind a flag, so the fallback remains necessary ([MDN format guide](https://developer.mozilla.org/en-US/docs/Web/Media/Guides/Formats/Image_types)).
