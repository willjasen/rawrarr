# Import folded age-encrypted root backups

## Objective and initial state

Diagnose and fix the `/setup/node` root-backup import error while keeping age identities and backup contents in browser memory. The live page showed the generic unlock error even with the expected identity. Inspection found the complete age armor header and footer, but its base64 body line separators had been converted into spaces; the browser parser expected the age armor's newline-delimited 64-character lines.

## Change

- Normalize a syntactically valid age armor body by removing whitespace and wrapping base64 at 64 characters before browser-side decryption. Invalid headers, footers, or base64 are still rejected by the age parser.
- Apply the same normalization to root-backup inspection and signing.
- Extend the browser test to generate an age identity and ML-DSA-65 root, import the identity, then import a whitespace-folded encrypted root backup. Extend crypto coverage for folded-backup inspection and signing.

## Verification and deployment

- The Playwright test generates a synthetic hybrid age identity and ML-DSA-65 signing root, imports the generated age ID, and imports a folded backup through `/setup/node`; it passed in `rawrarr-testing-base:local`.
- The focused crypto tests passed with `node --experimental-global-webcrypto --test scripts/tests/test_rawrarr_device_signing_crypto.mjs`. The first invocation without that Node flag failed because the test image did not expose Web Crypto globally; rerunning with the flag passed.
- Rebuilt `device-signing-client.js` and `subscriber-onboarding.js`, then rebuilt and recreated only `roarlay` in Compose project `rawrarr-orbstack-test`.
- Test coordinator health returned HTTP 200. Tailscale Serve still maps `rawrarr-orbstack-test-coordinator.risk-mermaid.ts.net:443` `/` to `http://127.0.0.1:8080`. The HTTPS-served onboarding bundle SHA-256 matches the rebuilt local bundle.
- PostgreSQL, Headscale, and both metadata nodes remained healthy and running. No production container or Serve route was changed.

The already-open browser page still has the old JavaScript loaded. Reload it after ensuring the saved backups are available, then re-import the age ID and encrypted root backup. The age identity and backup remain browser-only.

## Recovery

Revert only the task-owned source, generated asset, test, and runbook changes to restore strict age armor line formatting. Do not remove or reset test stack volumes or Tailscale identities.
