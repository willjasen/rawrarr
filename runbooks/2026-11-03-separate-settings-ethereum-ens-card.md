# Separate the Settings Ethereum and ENS card

## Objective

Move the account tier, linked ENS wallet status, wallet-link form, and subscription link into a dedicated card on the coordinator Settings page. Tracked by issue #363.

## Implementation

- Keep the profile image, display name, profile-picture status, and trial status in the profile card.
- Add an Ethereum & ENS card and retain the existing wallet form IDs and behavior.
- Use a responsive grid so cards share a row when there is room and stack on narrow screens.

## Compatibility and adoption

This is a static coordinator UI change; it does not change persistent data, APIs, schemas, encrypted configuration, or node protocols. No migration is required. Existing deployments adopt the layout when the updated coordinator UI is published.

## Verification

- Run the isolated regression suites with `python3 scripts/tests/run_rawrarr_tests_in_docker.py`.
- Playwright coverage checks card ownership, wallet/subscription controls, and desktop/mobile overflow.
