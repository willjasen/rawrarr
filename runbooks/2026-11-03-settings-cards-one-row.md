# Put Settings cards on one row

## Objective

Arrange the profile, Ethereum & ENS, and browser age-ID cards in one row on desktop, while keeping them stacked and usable on narrow screens. Tracked by issue #363.

## Implementation

- Use three equal, shrinkable columns in the existing Settings identity grid.
- Keep the existing single-column breakpoint for narrow viewports.
- Extend the Settings Playwright regression test to verify card order, shared desktop row, mobile stacking, and control bounds.

## Compatibility and adoption

This is a static coordinator UI layout change. It does not change persistent data, APIs, schemas, encrypted configuration, or node protocols, so no migration is required. Existing deployments adopt the layout when the updated coordinator UI is published.

## Verification

- Run the isolated regression suites with `python3 scripts/tests/run_rawrarr_tests_in_docker.py`.
- The Settings Playwright test checks desktop (1280×800) and narrow mobile (375×812) geometry and horizontal overflow.
