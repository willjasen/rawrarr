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

PR #291 merged to `main` as `0a7137bd846dc96013ee2bc7caa46598f0e0d85f`. PR #292 added desktop and mobile Playwright coverage and fixed narrow-screen wrapping; it merged as `c001b779a31da9ecbee0f3cdae1935a2bb94773b`.

On 2026-10-28, the exact published `c001b779a31da9ecbee0f3cdae1935a2bb94773b` archive was verified by SHA-256 (`bfb10fd5b90404ec6ea1f7e599fdbad76163e2d4d185323df0928e6db1d12f3c`) and built locally on each target host. The deployment classifier selected both coordinator and node targets. Updated only the four affected application services:

- do3 internal coordinator `rawrarr-internal-do3-roarlay`, native `linux/amd64`, image `sha256:81fd41b481e88a7631d9cb7149ec68579fb3aeaa02e875b2e9b792e0a3f936e7`.
- do3 public coordinator `rawrarr-roarlay-prod-roarlay`, native `linux/amd64`, using the same image.
- Brandon's MacBook Pro `rawrarr-node-internal-fossil-1`, native `linux/amd64`, image `sha256:668847f5876f1905e77c89189dab3c003431a36ec64507ff2c5f4f99dbf73349`.
- Brandon's Mac mini `rawrarr-internal-fossil-1`, native `linux/arm64`, image `sha256:a9ed578309c31e8119346ec7d970444f441f5750d88918c2383a75c4c1239460`.

All four services reported healthy. Both Mac fossil `/health` routes returned the Rawrarr System health page. Internal `https://rawrarr.risk-mermaid.ts.net/health` succeeded; Tailscale Serve read-back confirmed `/` on HTTPS 443 still proxies to `http://127.0.0.1:8080`. Public `https://rawrarr.willjasen.net/health` succeeded. The internal and public `/billing` pages served the Subscription content. Existing data volumes and dependent services were not recreated. Previous do3 and Mac base images remain available under host-local rollback tags.

The shared checkout's `main` branch was left untouched because it had unrelated unresolved analytics changes; deployments used only the exact published GitHub archive.

## Rollback

Revert the task commit to restore the prior Settings Stripe controls and Stripe return URLs. No persistent data was modified.
