# Implement browser mini metadata export

## Objective and initial state

Implement a testable browser proof of concept that exports a small copy of encrypted movie metadata and matching artwork. The existing service keeps the age identity and decrypted config in its own unlocked vault process; this export must independently decrypt in the browser without sending the user's ID or HMAC secret to the service.

## Actions

- Added an authenticated, ciphertext-only API for an index limited to three movie text records, a bounded list of opaque artwork filenames, the encrypted shared config, and validated ciphertext paths under the metadata directory.
- Added a browser bundle using the pinned `age-encryption` package for post-quantum age decryption and `fflate` for a local ZIP. The browser decrypts the shared config, obtains its metadata HMAC key, checks text HMAC names against decrypted TMDb IDs, finds matching artwork names, and writes stable-ID paths plus a manifest.
- Added a one-use age-ID dialog in Movies. It clears the input after use and presents generic crypto errors to avoid echoing malformed private IDs.
- Added a Playwright test using ephemeral age-keygen/age CLI ciphertext, including wrong-key handling, HMAC verification, archive contents, and a check that the private ID never appears in requests.

## Verification

- Browser mini-export Playwright test passed in the isolated test image using real ML-KEM-768 + X25519 age ciphertext from age v1.3.2.
- Rust filename-validation test passed in the isolated test image.
- Rebuilt the existing `rawrarr-local` metadata service on 127.0.0.1:7297 and verified `/health` returned HTTP 200, the service was healthy, and the browser bundle SHA-256 matched the source. The export index returned HTTP 401 without a session. Compose also recreated the coordinator because both services share the `rawrarr:local` image; coordinator and metadata are healthy. No volumes were removed.
- A follow-up build after the final dialog/cache-header polish failed on the pre-existing coordinator initializer at `rust-arr/src/coordinator.rs:432`: it omits the newly required `AuthState.next_path`. That unrelated code was left untouched; the already deployed core mini-export remains healthy and testable.

## Final state and remaining work

The three-record proof of concept is available for local UI testing at `http://127.0.0.1:7297/#movies` after signing in. Full export scaling, other metadata domains, cancellation, and comprehensive partial-failure reporting remain planned.
