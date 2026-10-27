# Add test-mode account checkout for Stripe

## Objective

Let an authenticated account on `rawrarr.willjasen.net` start a Stripe Test-mode subscription at a temporary $1/month USD price and see its webhook-reported status. Keep card entry on Stripe Checkout, keep the Stripe API key server-side, and do not alter trial access or grant product entitlements.

## Initial state

- Issue [#160](https://github.com/willjasen/rawrarr/issues/160) tracks Stripe billing. PR [#232](https://github.com/willjasen/rawrarr/pull/232) is merged. The do3 webhook service and a prior-account test endpoint were already deployed; this continuation connects the new Rawrarr Stripe account. See [the do3 deployment runbook](2026-10-27-deploy-stripe-service-do3.md) for the earlier endpoint setup.
- At the start of implementation, the coordinator authenticated users with `rawrarr_session` and exposed public account trial dates, but had no authenticated Checkout action or billing status view.
- The subscription model proposes one subscription per shared instance, but instance ownership, live pricing, billing portal behavior, and paid entitlements remain undecided. This task uses a temporary Test-mode price only and must not represent it as live pricing.

## Scope and safety

- Use a temporary $1/month USD Stripe Test-mode Price. Do not create or use a live-mode price or key.
- Require the existing coordinator account session for Checkout and status operations. Never accept an account ID from the browser as authoritative.
- Keep the Stripe API credential in a host-mounted secret file. Do not expose it to browser code, subscriber nodes, repository files, or logs.
- Treat Stripe webhooks as the source of subscription state. A successful Checkout redirect alone is not proof of payment.
- Do not activate entitlements, enforce trial expiry, or alter Rawrarr data access in this task.
- Link the existing Stripe webhook service deployment rather than changing unrelated containers.

## Actions

Implemented a public coordinator-only, session-authenticated Stripe Test-mode Checkout and portal slice. The coordinator requires a `sk_test_` key and a configured Price that Stripe confirms is active, Test mode, USD 100 cents, monthly, interval count 1. Checkout binds the authenticated account ID into session and subscription metadata; customer/subscription status is read from the webhook service's account-linked tables. The browser receives only allowlisted Stripe-hosted URLs. The Settings copy explicitly says this does not change access or entitlements. The do3 overlay mounts key and price ID as host files; no secret value is in the repository.

Created the original-account Stripe Test-mode product `Rawrarr Test Subscription` with temporary $1.00 USD monthly Price `price_1UVJ6CRLykSbVssA2E4CDknp`. After switching to the new Rawrarr Stripe account, created and verified its corresponding active $1.00 USD/month Price `price_1UVJgAEWAWlyUzCFW5VKPaCZ`; the do3 price-ID file now points to this Price. These IDs are not secrets.

Added static regression coverage in `scripts/tests/test_rawrarr_stripe_checkout.mjs`. The full isolated test results are recorded below. The host Mac Rust compiler could not start because the installed LLVM toolchain could not load `libz3.5.1.dylib`; the isolated OrbStack runner was used for verification.

## Verification and current state

Local verification: `git diff --check` passed; all 12 focused Stripe and trial Node.js tests passed; `cargo check --locked --offline` passed in the isolated Rust container; all three focused `coordinator::stripe::tests` passed. After rebasing onto current `main`, the full isolated runner passed all 130 Node.js tests, all Playwright browser tests, and all 111 Rust unit tests. Its Python integration tests are intentionally skipped because they require disposable PostgreSQL, age, or Syncthing fixtures. Rustfmt was unavailable in both local and isolated toolchains (the host Rust installation cannot load its required `libz3.5.1.dylib`; the test image lacks the component and could not download it because container DNS failed).

The published `main` revision `e229765b00453eda7c3bca1fd6e4d7f47dc967db` was built on do3 as `rawrarr:do3-e229765` and deployed to `roarlay`. The image reports the expected `linux/amd64` architecture and revision. The service is healthy; public `/health` and `/billing` both return HTTP 200. PR #232 merged as `a9e434995bc55d725ab6c6a2db2bf83f9d12b3e5`.

The currently installed API key authenticates to Rawrarr's Stripe Test-mode account `acct_1UVJYjEWAWlyUzCF`. A later key paste was rejected by the hidden installer because it did not start with `sk_test_`; the previously installed key remains in place and was revalidated without displaying it. Its host file remains UID/GID 10001, mode 0400.

Created a Test-mode webhook endpoint for the Rawrarr account, `we_1UVJn7EWAWlyUzCFdujYq485`, at `https://rawrarr.willjasen.net/v1/webhooks/stripe`, API version `2025-05-28.basil`, with the ten issue #160 event types. Installed the endpoint's signing secret directly at `/opt/rawrarr-roarlay-prod/secrets/stripe-webhook-secret` (UID/GID 10001, mode 0400), then force-recreated only `stripe-service` with `--pull never --no-deps`. The container is running with restart count 0 and mounts the protected secret file. A synthetic, correctly signed non-billing event returned HTTP 204 from the public route; an invalid signature returned HTTP 400. The endpoint is enabled in Test mode and Stripe's API confirms all ten event types.

The authenticated browser flow successfully opened Stripe Checkout for the temporary $1/month test subscription. Checkout currently displays the Stripe account's merchant name as `nerdhirn`, so no payment was submitted. No Stripe Dashboard event delivery, mapped billing event, live payment, or access/entitlement change has been exercised. The test checkout needs the account's public-facing merchant name corrected before Will completes a test payment and confirms webhook-backed subscription status.

## Remaining work

- Correct the Stripe account's Checkout-facing merchant name from `nerdhirn` to `Rawrarr` in Stripe account settings.
- Complete one Test-mode Checkout using the authenticated Rawrarr account, then verify Stripe's Dashboard delivery and the webhook-backed billing status. Do not use a live key or price.
- Keep live payments and entitlement changes disabled until the remaining issue #160 requirements are agreed and tested.
