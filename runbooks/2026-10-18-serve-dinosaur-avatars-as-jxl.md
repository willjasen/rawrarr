# Serve dinosaur avatars as JPEG XL

## Objective and initial state

The profile avatar generator ran in the browser and assigned an SVG data URL to the image element. The user requested that the coordinator serve raster JPEG XL instead, without exposing SVG markup to the browser or storing generated images in PostgreSQL.

## Actions

- Moved deterministic avatar generation to the coordinator. The browser fetches the authorized public age recipient through the existing relay, hashes it with SHA-256, and requests the image by digest from an authenticated workspace route.
- The coordinator creates its deterministic SVG internally, rasterizes it with `rsvg-convert`, and encodes lossless JPEG XL with `cjxl`. It writes only short-lived files in a unique temporary directory and removes that directory after the response is built.
- Added a PNG endpoint and `<picture>` fallback for browsers without JPEG XL support. Installed `librsvg2-bin` and `libjxl-tools` in the runtime image.
- Added Rust coverage for valid digest format, stable scenes, traits, rarity, wing/spike rules, JPEG XL codestream output, and PNG fallback. Updated design and test indexes.

## Verification

- `node --test --test-name-pattern='signed-in profile' scripts/tests/test_rawrarr_coordinator_workspace.mjs`: 1 passed. `node --check rust-arr/src/coordinator-workspace.mjs` and `git diff --check` passed.
- `cargo test coordinator_avatar::tests --bin rust-arr` with Rust 1.85.1: 5 passed, including JPEG XL codestream and PNG signature checks.
- Confirmed ports 7297 and 7298 were published by the existing `rawrarr-macbook-tailscale` container. Rebuilt only `roarlay` in `rawrarr-local` after claiming its deployment lock.
- After deployment, `rawrarr-local-roarlay-1` was healthy and local/tailnet `/health` both returned HTTP 200. Anonymous GETs to both image formats returned HTTP 401. Tailscale Serve still routes `/` to `http://127.0.0.1:8080` and `/direct-api` to `http://127.0.0.1:8082`. The deployed browser module contains no SVG avatar generator. The runtime container has `rsvg-convert` 2.54.7 and `cjxl` 0.7.0; an in-container SVG-to-PNG-to-JXL smoke conversion produced the JPEG XL codestream signature.
- An initial live check exposed missing per-handler session checks on the image endpoints; both handlers were updated to require an authenticated coordinator session and redeployed before completion.

## Final state and remaining work

- The authenticated coordinator generates and serves deterministic JPEG XL avatars, with PNG fallback. SVG markup is generated only server-side and temporary files are cleaned up. Neither images nor seeds are persisted in PostgreSQL. Anonymous access is rejected. A signed-in browser fetch of the image response was not exercised in this turn.
