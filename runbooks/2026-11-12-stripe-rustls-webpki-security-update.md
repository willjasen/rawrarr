# Stripe service rustls-webpki security update

Issue: [#456](https://github.com/willjasen/rawrarr/issues/456)

## Scope and compatibility

Update the Stripe service lockfile from `rustls-webpki` 0.103.15 to 0.103.16, the fixed release identified by the reported eMudhra intermediate-key security advisory. A regression test requires this exact patched version.

This dependency-only change does not alter persistent data, database schemas, encrypted configuration, or node/coordinator protocols. Existing deployments adopt the fix when the Stripe service image is rebuilt from the published source and only the `stripe-service` container is recreated. Do not roll out this local branch; use the exact merged `main` revision and the established Stripe deployment safeguards in [the deployment runbook](2026-10-27-deploy-stripe-service-do3.md).

## Verification

- `node --test scripts/tests/test_rawrarr_stripe_service.mjs` passed in the isolated OrbStack test clone (6 checks).
- `cargo test --locked --offline --manifest-path /workspace/stripe-service/Cargo.toml` passed in the isolated OrbStack test clone (7 tests); the build compiled `rustls-webpki` 0.103.16.
- `git diff --check` passed.
- Direct host test execution was unavailable because Node.js is absent from the host `PATH` and the host Rust toolchain's Homebrew LLVM cannot load its Z3 library; the focused checks passed in the isolated clone instead.
- Not deployed; deployment was not requested.

## Rollback

No data rollback or migration is needed. If the rebuilt service cannot start, retain the previous image and service state for recovery, but report the security tradeoff before restoring the image with the vulnerable dependency.
