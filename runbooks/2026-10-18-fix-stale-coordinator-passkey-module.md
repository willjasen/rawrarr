# Fix stale browser module on coordinator passkey page

## Objective and initial state

The coordinator passkey page remained at its initial status. The browser console showed that `coordinator-passkey.mjs` imported `encodeCredential` from a cached older `/assets/passkey-client.mjs` that did not export it, although the live server returned the new export.

## Actions

- Added a version query to the coordinator page's shared passkey-client module import to invalidate older browser module cache entries.
- Changed the initial page status to explain how to recover if module startup fails before the page script can update the status.
- Changed registration to prepare the challenge first, then wait for an explicit **Create coordinator passkey** click before calling `navigator.credentials.create`. The page now explains that the browser's passkey prompt is waiting.

## Verification

- Focused Node tests passed (9 tests); JavaScript syntax and `git diff --check` passed.
- The disposable role integration passed with a headless Chromium virtual authenticator after updating the helper to click the new button; the coordinator credential was added to the same account in the fixture database.
- After the concurrent coordinator rebuild completed, live checks confirmed the coordinator is healthy, Tailscale health returns HTTP 200, the passkey page contains the new button, the live module imports the versioned client URL, and that module exports `encodeCredential`.
- Live inspection confirmed the reported link had already created and consumed its registration challenge. That link is not reusable.
- A later read-only database check showed two passkeys associated with one user and no outstanding registration challenge, confirming a coordinator credential is now stored on the existing account. Credential contents were not inspected.

## Final state

The fix is deployed and the coordinator account currently has two credentials owned by the same user. The page now gives an explicit button and status before the browser ceremony.
