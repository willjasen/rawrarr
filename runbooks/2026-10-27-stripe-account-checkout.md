# Add test-mode account checkout for Stripe

## Objective

Let an authenticated account on `rawrarr.willjasen.net` start a Stripe Test-mode subscription at a temporary $1/month USD price and see its webhook-reported status. Keep card entry on Stripe Checkout, keep the Stripe API key server-side, and do not alter trial access or grant product entitlements.

## Initial state

- Issue [#160](https://github.com/willjasen/rawrarr/issues/160) tracks Stripe billing. The webhook service and test endpoint are deployed on do3; the test signing secret is installed and the public route/signature checks passed. See [the do3 deployment runbook](2026-10-27-deploy-stripe-service-do3.md).
- The coordinator already authenticates users with `rawrarr_session` and exposes the public account trial dates. There is no authenticated Checkout action or billing status view yet.
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

Created Stripe Test-mode product `Rawrarr Test Subscription` with temporary $1.00 USD monthly Price `price_1UVJ6CRLykSbVssA2E4CDknp`. Stripe Dashboard showed the product active, the price $1.00 USD per month, and successful product/price creation responses. This Price ID is not a secret.

Added static regression coverage in `scripts/tests/test_rawrarr_stripe_checkout.mjs`. Runtime compile/tests and deployment are pending. The host Mac Rust compiler cannot start because the installed LLVM toolchain cannot load `libz3.5.1.dylib`; use the isolated OrbStack test runner as the established verification path.

## Verification and final state

Local verification: `git diff --check` passed; all 12 focused Stripe and trial Node.js tests passed; `cargo check --locked --offline` passed in the isolated Rust container; all three focused `coordinator::stripe::tests` passed. The full isolated runner passed all 130 Node.js tests. Its Playwright browser suite reported 11 failures in existing artwork timing, movie identifiers/links, library responsiveness/snapshot, navigation, Sonarr settings, and topology recovery tests; these are outside the Stripe code paths. The runner's first Rust suite attempt caught a missing `Zeroizing` import in the new test module, which was fixed; the three focused Rust tests then passed. A full Rust suite has not yet been rerun after that fix. Rustfmt is unavailable in both local and isolated toolchains (the host Rust installation cannot load its required `libz3.5.1.dylib`; the test image lacks the component and could not download it because container DNS failed).

No Checkout session, subscription webhook, Stripe dashboard delivery, live payment, or access/entitlement change has been exercised yet. The account flow still needs deployment configuration and a user-initiated test Checkout to verify webhook-backed status end to end.

## Remaining work

- Compile and run the new and existing tests in the isolated runner; fix any failures.
- Merge/publish the implementation, then install the do3 Test-mode API key and temporary Price ID.
- Test Checkout and the resulting webhook delivery with Will's own authenticated account; keep live payments disabled.
