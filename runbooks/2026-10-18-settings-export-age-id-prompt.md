# Prompt for age ID from Settings metadata export

## Objective and initial state

Fix the coordinator Settings export action when no age identity is currently in tab memory. The action previously displayed an instruction to unlock the ID in Movies, although users could reach Settings without doing so.

## Actions

- Added a one-use password dialog in Settings to accept the post-quantum age private ID and start export directly.
- Kept the ID in JavaScript memory for the active tab only; cleared the dialog field on submit and close. Existing browser-side decryption and ciphertext-only network requests remain in use.
- Added a focused coordinator workspace source test for the no-key prompt and cleared input behavior.

## Verification

The focused Node test passed. JavaScript syntax checks passed. The wider coordinator workspace test file has one unrelated existing assertion failure: it expects the age-ID copy "kept only in this tab's memory", but the current page copy was changed to the browser-profile wording.

## Final state

The Docker build completed with one existing dead-code warning. Focused coordinator workspace test and JavaScript syntax checks passed. Roarlay is healthy, HTTPS `/health` returns HTTP 200, the deployed workspace module contains the age-ID prompt, and Tailscale Serve still proxies `/` to `http://127.0.0.1:8080`. The deployment used a temporary Compose override to share the existing `rawrarr-macbook-tailscale` network namespace because the base file binds an already-owned host port. The checkout has broad pre-existing modifications; no commit was made from mixed-ownership files.
