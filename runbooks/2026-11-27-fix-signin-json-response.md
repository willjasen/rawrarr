# Fix sign-in JSON response handling

Issue: [#476](https://github.com/willjasen/rawrarr/issues/476)

## Objective and initial state

Fix the sign-in screen's raw JSON parse error when an API failure returns an empty or non-JSON body. The screenshot showed this during ENS wallet sign-in. The page unconditionally called `Response.json()`, while some expected failures (including an unavailable/disabled wallet route) can have no JSON body. The main checkout was clean at `ab53500`; implementation is isolated in `codex/fix-signin-json-response`.

## Changes

- Added a shared browser response parser that handles JSON API errors, empty/non-JSON HTTP errors, invalid successful JSON, and empty successful responses with useful messages.
- Served the helper from both the subscriber and coordinator asset routes and used it for sign-in POST requests.
- Added regression cases for empty body, non-JSON body, and server-provided JSON error.
- No persistent data, protocol, or compatibility changes; no migration is required.

## Verification

- `node --test scripts/tests/test_rawrarr_passkey_ui.mjs` — passed (5 tests).

## Final state and remaining work

Local implementation and focused test are complete. No deployment or live wallet sign-in verification was performed. Rust compilation and browser-level behavior were not verified.
