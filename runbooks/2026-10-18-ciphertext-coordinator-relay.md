# Ciphertext-only coordinator relay

## Objective and initial state

Replace the coordinator's general node-page relay with browser-side rendering over a ciphertext-only relay, and keep the age private ID in browser tab storage. The checkout already contained a three-record browser decryption/ZIP proof of concept and extensive unrelated changes. The coordinator page relay previously forwarded arbitrary HTML and API responses.

## Actions

- Restricted coordinator node relay requests to `GET` of the bounded metadata export index, encrypted config, and validated movie ciphertext objects. General node pages and plaintext API routes now return 404.
- Added browser-side metadata decryption and rendering, including poster images, with the age identity held in tab-scoped `sessionStorage` and cleared on sign-out or explicit user action.
- Kept metadata ciphertext source on the metadata node and used the coordinator as an authenticated relay; no private age identity is sent in requests.
- Fixed the follow-up browser error: the bundle is a self-executing script that installs `window.rawrarrMiniExport`, not an ES module with named exports. The workspace now imports it for side effects and calls the installed browser API.
- Added focused coordinator relay allowlist coverage and updated architecture/status/deployment docs.

## Verification

The focused Node coordinator workspace test passed. The isolated test runner passed all 37 Rust tests and its Playwright age setup check. Eight unrelated Node tests still fail: age setup VM fixture, four artwork timing fixtures, lowercase naming, title slug, and shared library size formatting. Compose config validation and `git diff --check` passed. Rebuilt only `rawrarr-local-coordinator-1`; it is healthy. After fixing the bundle API call, the live workspace asset references `window.rawrarrMiniExport.loadMiniMetadata`, and the served decryption bundle contains that function. HTTPS coordinator `/health` returned HTTP 200. Tailscale Serve still maps `/` to `http://127.0.0.1:8080`. Both metadata and storage heartbeat timestamps were fresh. Unauthenticated legacy page and ciphertext relay requests both returned HTTP 401; the Rust relay allowlist tests confirm only expected ciphertext paths are accepted after authentication.

## Recovery and remaining work

The prior image remains available through Docker image history for rollback; no data volumes or shared ciphertext were changed. The coordinator relay is stateless: it requests selected ciphertext from the metadata node and streams the bytes to the browser; it does not persist uploaded objects. The bounded browser view covers only three movie records. Full-library inventory, explicit storage-node publishing, and all metadata domains remain future scope. Task-only files could not be committed because this checkout already has broad mixed changes.
