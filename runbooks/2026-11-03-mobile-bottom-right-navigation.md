# Mobile bottom-right navigation

## Objective and issue

Move the subscriber workspace's top-row navigation into a bottom-right menu on mobile, matching the fossil-node view in [issue #399](https://github.com/willjasen/rawrarr/issues/399). Keep desktop navigation and the separate coordinator workspace unchanged.

## Design and compatibility

- The subscriber workspace continues using its existing `showView` logic for active navigation state, hash URLs, refresh, and browser history.
- At widths of 580px and below, the existing navigation items are hidden until opened from the fixed bottom-right menu. The menu closes after selection, on Escape, and on outside interaction.
- The menu toggle exposes its expanded state and accessible name; the closed navigation is inert and hidden from assistive technology. Its placement and reserved page-bottom space account for mobile safe-area insets.
- This is a static subscriber UI change. It has no persisted-data, database-schema, encrypted-configuration, or node/coordinator protocol impact, so existing deployments need no migration. Roll out the published UI to affected subscriber services only; the coordinator is unaffected.

## Verification

- `scripts/tests/test_rawrarr_navigation.mjs` passed: menu visibility, all destinations, selection, hash/active state, Escape and outside dismissal, keyboard focus restoration, horizontal overflow, mobile placement at 375×812, and desktop visibility at 1280×800.
- `python3 scripts/tests/run_rawrarr_tests_in_docker.py` passed: 155 Node.js tests, all Playwright tests, and 123 Rust tests. Python integration tests requiring disposable PostgreSQL, age, or Syncthing fixtures were skipped by the runner.

## Rollback

Revert the focused UI, test, and runbook changes in a reviewed release. No data restoration or migration rollback is required.
