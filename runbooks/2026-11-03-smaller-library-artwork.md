# Show more library artwork per row

## Objective

Reduce the minimum width of shared Movies and Sonarr poster cards so more artwork appears on each desktop row. Tracked by issue #395.

## Implementation

- Reduce the shared library grid's desktop card minimum from 174px to 140px.
- Preserve the existing two-column mobile layout.
- Extend the movie artwork browser regression to verify six columns on desktop and two on mobile; this grid is shared with Sonarr.

## Compatibility and adoption

This is a presentation-only coordinator UI change. It does not change persistent data, APIs, schemas, encrypted configuration, or node protocols, so no migration is required. Existing deployments adopt the layout when the updated coordinator UI is published.

## Verification

- `python3 scripts/tests/run_rawrarr_tests_in_docker.py` — passed; Rust, Node.js, and Playwright suites reported exit code 0.
- The artwork browser test checks 1280×800 and 375×812 viewports, shared grid column counts, and horizontal overflow.
