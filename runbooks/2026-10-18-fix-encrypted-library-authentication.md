# Fix encrypted library response authentication

## Objective and initial state

Restore the Movies page after the browser reported that the encrypted Radarr library response failed AES-GCM authentication. The coordinator and metadata relay were healthy; source inspection showed that the metadata node derives its response key using HKDF-SHA-256, while the browser used the raw ECDH secret directly as an AES-GCM key.

## Actions

- Changed browser key agreement to derive the 256-bit ECDH shared secret, then derive the AES-256-GCM key with HKDF-SHA-256 using the same salt and info strings as the metadata node.
- Granted the generated ECDH private key the `deriveBits` usage required by that flow; Chrome surfaced the usage mismatch after the first protocol correction.
- Zeroed temporary shared-secret and HKDF parameter byte arrays after key derivation.
- Added a focused protocol consistency assertion to the coordinator workspace test.
- Rebuilt the embedded browser bundle and deployed the coordinator image.

## Verification

- The focused coordinator workspace Node test passed all 6 checks; `git diff --check` passed.
- Rebuilt and recreated only the coordinator container. The coordinator and metadata recovery containers report healthy; local metadata `7297`, local coordinator `7298`, and tailnet `/health` all returned HTTP 200.
- Read back Tailscale Serve configuration and confirmed `rawrarr.risk-mermaid.ts.net:443` `/` still proxies to `http://127.0.0.1:8080`.
- Reloaded the signed-in Chrome tab and verified the usual Movies page rendered with 1,941 decrypted Radarr entries, summary counts, the movie table, and artwork. The age identity remained in browser tab storage and was not exposed in inspection output.
- Removed the coordinator deployment marker after health and route verification.

## Recovery and remaining work

If the page still fails, preserve the current key agreement code and inspect the browser network response length and active metadata node image/version before changing protocol parameters. The authenticated Radarr response must remain encrypted end to end through the coordinator.
