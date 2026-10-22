# Age-wrapped browser response session

## Objective and initial state

Use the user's public age recipient to wrap a fresh symmetric response key for each browser request. The metadata node must unwrap that key locally and encrypt its response so the coordinator can only forward opaque bytes. Existing Radarr library responses used ephemeral P-256 ECDH; the coordinator already allowed only that bounded request body for its relay. Work is isolated in the `age-session-response` worktree from `main`.

## Actions

- Changed the browser to fetch the metadata node's public recipient, generate a random 256-bit AES key, age-encrypt it, and POST the armored key to `/api/metadata/session/library`.
- Changed the metadata node to unwrap the key with its node-local age identity using the existing protected subprocess helper, then encrypt the JSON response with AES-256-GCM. The response is nonce followed by ciphertext/tag.
- Changed the coordinator allowlist to accept only bounded age-armored payloads for that POST; it still relays the response bytes without decryption.
- Updated the embedded browser bundle and focused source/relay assertions.

## Verification

- `node --check rust-arr/web-export/metadata-export.js` and `node --check rust-arr/src/assets/metadata-export.js` passed.
- The updated focused browser assertion passed within `node --test scripts/tests/test_rawrarr_coordinator_workspace.mjs`.
- The full browser suite reports 7 failures in existing expectations for unrelated coordinator page/workspace behavior; 13 of 20 tests pass.
- Rust tests could not start because the host `rustc` aborts while loading LLVM due to missing `/usr/local/opt/z3/lib/libz3.5.1.dylib`.
- `npm run build` could not run because the worktree has no installed `esbuild` binary; the source change was reflected in the embedded bundle directly and both files pass syntax checks.
- Initial internal rollout used the task commit directly before the GitHub-first deployment instruction was added. Roarlay rebuilt successfully in `rawrarr-internal-mini`; image `sha256:7bb53c97907e3e1b72f725f249dc1b39a6767869d9bd0cb6884114c7476f43f6` became healthy with zero restarts. HTTPS `/health` returned HTTP 200 and Serve remained `/` -> `http://127.0.0.1:8080`.
- At the user's direction, the task commit was replayed on current `origin/main` as a focused branch. The mini source is being updated from that published GitHub revision and roarlay rebuilt again to establish GitHub as the deployment source.

## Security notes and remaining work

The encrypted response protects content from the coordinator, but not request timing, response size, or node/account association. The node identity remains node-local; requests fail closed if it is unavailable or does not decrypt exactly a 32-byte key. Browser JavaScript key bytes are cleared after importing into a non-extractable Web Crypto key. Re-run the Rust round-trip test and regenerate the bundle in an environment with the project toolchain and npm dependencies. No deployment was performed.
