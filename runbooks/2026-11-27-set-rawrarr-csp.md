# Set Rawrarr CSP

Issue: [#492](https://github.com/willjasen/rawrarr/issues/492)

## Objective and initial state

Address the CSP finding shown for `https://rawrarr.net` and ensure the Rawrarr Rust application also sends a CSP. The public static site already returned a CSP on 2026-11-27 (`curl -fsSI https://rawrarr.net/`); `netlify.toml` defines it. Rust responses had no common security-header middleware, though one coordinator passkey page set a page-specific CSP.

## Actions

- Added common Rust response middleware setting CSP and `X-Content-Type-Options: nosniff`.
- Policy blocks objects and framing, restricts sources to same-origin by default, allows existing inline UI script/style blocks, app API connections over HTTPS/WSS, and the TMDb logo.
- Added focused regression coverage and indexed its invocation in `scripts/tests/README.md`.

## Compatibility and adoption

No persisted data, database, encrypted configuration, or node/coordinator protocol changes. Deploy the normal published Rawrarr application image to adopt the response headers. The public static site's configured CSP is already active; its scan can be rerun after the crawler refreshes.

## Verification and final state

- `node --test scripts/tests/test_rawrarr_security_headers.mjs` passed.
- `git diff --check` passed.
- `node --test scripts/tests/test_public_landing_page.mjs` could not start because `/usr/local/lib/node_modules/playwright-core/index.mjs` is missing in this host context.
- `cargo test --locked --manifest-path rust-arr/Cargo.toml --no-run` is still compiling the crate; completion is pending. `cargo fmt --check` reports extensive pre-existing formatting differences across unrelated source and fixture files, so no broad formatting was applied.
- Live response check: `https://rawrarr.net/` returns a CSP. `https://roarlay.rawrarr.net/` currently returns a 303 redirect without CSP; this will be covered by the coordinator middleware after deployment.

No deployment performed; published-source deployment and live verification remain follow-up work.
