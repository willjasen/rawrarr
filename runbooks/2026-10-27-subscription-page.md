# Move ETH and Stripe billing into the Subscription page

## Request and tracking

Move ETH donation and subscription controls out of Settings into a dedicated Subscription page, with Stripe billing on that page too. The existing `/billing` route already held the ETH flows, so it now serves as the combined page. Settings keeps the account tier and ENS wallet linking controls and links to `/billing`.

GitHub issue [#290](https://github.com/willjasen/rawrarr/issues/290) tracks this change and its acceptance criteria. It was created after authenticated GitHub access became available; the original implementation attempt had recorded the temporary access limitation here.

## Changes

- Renamed and expanded `/billing` as the Subscription page and moved Stripe status, checkout, and portal controls there.
- Removed Stripe billing controls and their loading/interaction code from Settings. Settings retains account tier and ENS wallet controls and links to `/billing`.
- Updated Stripe Checkout, cancel, and portal return URLs to `/billing`.
- Added wrapping for the Ethereum recipient address so it does not force horizontal scrolling on narrow screens.
- No database, encrypted configuration, shared data, or protocol changes; no migration is needed. This is a presentation and navigation change.

## Verification

- `node --test scripts/tests/test_rawrarr_stripe_checkout.mjs` — passed (3 tests).
- `python3 scripts/tests/run_rawrarr_tests_in_docker.py --remove-after` — passed: 144 Node.js tests, all Playwright files (including 1280×800 and 375×812 Subscription page checks), and 113 Rust tests. Python integration fixtures were skipped by the runner as documented.
- `git diff --check` — passed.

## Deployment

PR #291 merged to `main` as `0a7137bd846dc96013ee2bc7caa46598f0e0d85f`. Deployment is pending the responsive Playwright follow-up and final published revision.

## Rollback

Revert the task commit to restore the prior Settings Stripe controls and Stripe return URLs. No persistent data was modified.
