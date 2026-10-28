# Move ETH and Stripe billing into the Subscription page

## Request and tracking

Move ETH donation and subscription controls out of Settings into a dedicated Subscription page, with Stripe billing on that page too. The existing `/billing` route already held the ETH flows, so it now serves as the combined page. Settings keeps the account tier and ENS wallet linking controls and links to `/billing`.

GitHub issue creation was attempted before implementation but is unavailable in this host context: `gh auth status` reports the saved `willjasen` token is invalid, and `gh issue list` could not connect to `api.github.com`. Proposed issue:

> **Title:** Move ETH and Stripe controls to a Subscription page
>
> **Acceptance criteria:** The coordinator has a dedicated Subscription page containing ETH donations, ETH subscriptions, and Stripe checkout/status/portal controls. Settings retains account tier and ENS wallet linking, with a link to the Subscription page. Stripe Checkout and portal return to the Subscription page. Existing billing, entitlement, and donation behavior is unchanged.

Create or link the issue when authenticated GitHub access is restored.

## Changes

- Renamed and expanded `/billing` as the Subscription page and moved Stripe status, checkout, and portal controls there.
- Removed Stripe billing controls and their loading/interaction code from Settings. Settings retains account tier and ENS wallet controls and links to `/billing`.
- Updated Stripe Checkout, cancel, and portal return URLs to `/billing`.
- No database, encrypted configuration, shared data, or protocol changes; no migration is needed. This is a presentation and navigation change.

## Verification

- `node --test scripts/tests/test_rawrarr_stripe_checkout.mjs` — passed (3 tests).
- `node --test scripts/tests/test_rawrarr_crypto_ui.mjs` — not run successfully; the host environment lacks `/usr/local/lib/node_modules/playwright-core/index.mjs`, required by the test. The isolated OrbStack Playwright suite was not run.
- `git diff --check` — pending final review.

## Rollback

Revert the task commit to restore the prior Settings Stripe controls and Stripe return URLs. No persistent data was modified.
