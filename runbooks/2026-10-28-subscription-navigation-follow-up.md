# Make Subscription discoverable in workspace navigation

Issue: [#290](https://github.com/willjasen/rawrarr/issues/290)

## Request

After the ETH and Stripe controls moved to `/billing`, the Settings page linked to it as “Manage subscription”, but the coordinator sidebar had no Subscription entry. The page existed but was not discoverable from the main navigation shown in the report.

## Change

- Added a Subscription link to `/billing` in the main workspace sidebar, near Overview, with an explicit accessible name for compact mobile navigation.
- Kept Settings' “Manage subscription” link as a second entry point.
- No storage, database, encrypted configuration, protocol, or existing-state changes; no migration is needed.

## Verification

- `node --test scripts/tests/test_rawrarr_coordinator_workspace.mjs` — passed (34 tests).
- `python3 scripts/tests/run_rawrarr_tests_in_docker.py --remove-after` — passed: 144 Node.js tests, all Playwright tests including Subscription navigation and `/billing` interaction at 1280×800 and 375×812, and 113 Rust tests. Python integration fixtures were skipped by the runner as documented.
- `git diff --check` — passed.

## Deployment

- Pending merge and deployment of the published revision to the internal and public coordinator services.

## Rollback

- Revert the navigation link and its regression test. The existing `/billing` route and Settings link remain functional.
