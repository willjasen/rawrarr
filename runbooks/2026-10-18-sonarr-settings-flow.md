# Finish Sonarr connection settings

## Objective and initial state

Make the Sonarr connection entry usable like Radarr: accept an HTTPS URL and API key, validate the service, save credentials in the subscriber's age-encrypted configuration, and preserve other app settings. The checkout contains broad concurrent changes; only Sonarr-focused test/index/runbook changes belong to this task. Related context: [Sonarr integration and shared library layout](2026-10-18-sonarr-shared-library.md).

## Implementation

- Reviewed the Sonarr form, authenticated settings routes, Sonarr service validation, and `AgeVault::save_sonarr` read-modify-write path already present in the checkout.
- Added a coordinator Settings form that submits age unlock and Sonarr settings only through the browser-to-metadata direct path. Roarlay receives neither the private age identity nor the Sonarr API key.
- Added Chromium regression coverage for direct route submission, the Sonarr settings form, successful save, URL read-back without returning the API key, password-field clearing, and connection clearing.
- Existing age-vault coverage checks encrypted save/clear, preservation of Radarr settings, and preservation of the metadata filename key.

## Verification

Passed in the isolated Rawrarr test image: 11 coordinator workspace checks; the coordinator Sonarr settings Chromium test; 5 focused Sonarr/config Rust tests; and 1 additional encrypted cross-app settings test. The broader all-tests command still reports failures in unrelated concurrent age-setup/artwork/node tests; its Node network-isolation test also expects Compose files the runner does not copy. Host Rust/Playwright remain unavailable because of the host LLVM/Z3 mismatch and missing Playwright package.

## Deployment and remaining work

Rebuilt metadata and roarlay in the existing `rawrarr-local` project. Both containers report healthy. Read back Tailscale Serve JSON: `/` still proxies to `http://127.0.0.1:8080` and `/direct-api` to `http://127.0.0.1:8082`; the public HTTPS health endpoint responds successfully. The roarlay rebuild lock was released after route and health checks.

A real Sonarr connection is not configured or verified yet. The user can enter the Sonarr HTTPS URL and API key in coordinator **Settings** after selecting an online metadata node and unlocking the age vault. The API key is sent to metadata over `/direct-api`, validated there, then stored in the age-encrypted config. The URL can be read back; the API key is not returned to the browser. The API-key field clears after save.
