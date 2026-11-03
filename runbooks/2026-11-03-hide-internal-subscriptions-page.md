# Hide the Subscription page on internal deployments

Issue: [#160](https://github.com/willjasen/rawrarr/issues/160)

## Change

- The coordinator serves `/billing` only when `RAWARR_BILLING_AUDIENCE=public`; internal or unset audiences receive HTTP 404, including direct requests.
- Internal coordinator workspaces omit the Subscription and Manage subscription links. Public billing behavior is unchanged.
- Existing internal Compose definitions already set `RAWARR_BILLING_AUDIENCE=internal`, while the public coordinator defaults to `public`.
- No database, synchronized data, encrypted configuration, protocol, or existing-state changes; no migration is required.

## Verification

- `node --test scripts/tests/test_rawrarr_stripe_checkout.mjs` — passed (4 tests).
- `python3 scripts/tests/run_rawrarr_tests_in_docker.py --remove-after` — passed: 155 Node.js tests, all Playwright tests, and 123 Rust tests. Python integration fixtures were skipped by the runner as documented.
- `git diff --check` — passed.

## Rollout and rollback

This is a coordinator application change; publish and deploy the exact reviewed revision to the internal coordinator service following the standard coordinator deployment procedure. No subscriber-node deployment or data migration is needed. Verify internal `/billing` returns HTTP 404 and the internal workspace has no subscription links, while the public `/billing` page remains available. Roll back by reverting the coordinator change and redeploying the prior published revision.
