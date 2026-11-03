# Fix mobile formatting on the Devices screen

Track the responsive layout fix in [issue #193](https://github.com/willjasen/rawrarr/issues/193).

## Change

At phone widths, Fossil cards now shrink to the available viewport width, route status labels wrap within their card, and card headings can place status below a long device name. Edit remains a comfortably tappable control. The page heading and Add a fossil link wrap without competing for a single row. Topology labels wrap and its outer nodes use narrower cards to stay inside the diagram.

The change affects only coordinator UI assets. It does not change persisted data, APIs, or node/coordinator protocols, so existing deployments need no migration; the coordinator adopts it with the normal application image update.

## Verification

- `node --test scripts/tests/test_rawrarr_coordinator_workspace.mjs` — passed (34 tests).
- `git diff --check` — passed.
- Desktop and narrow viewport visual checks: pending/not run in this environment.
