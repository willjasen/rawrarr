# Use ECDH for encrypted browser library sessions

## Objective and initial state

Integrate issue [#35](https://github.com/willjasen/rawrarr/issues/35): replace the age-wrapped session-key exchange used for the encrypted Radarr browser library response with an ephemeral P-256 ECDH exchange and HKDF-derived AES-GCM key. Keep age identities and age-encrypted metadata for the other features that use them.

Work is isolated in `codex/ecdh-browser-library-session`, based on GitHub `main` at `6814b3b`. The shared `main` checkout had staged, unstaged, and untracked work and was 31 commits behind GitHub; it was left untouched. The browser source and Rust node/coordinator still used the older age-wrapped response handshake on GitHub `main`.

## Changes

- Browser creates an ephemeral P-256 key pair, posts the 65-byte uncompressed public key, derives the shared secret from the node's ephemeral public key, and uses HKDF-SHA-256 with fixed protocol salt/info to derive an AES-256-GCM response key.
- Metadata node validates the browser public key, creates its own ephemeral P-256 key, derives the same key, and returns its public key followed by a random 12-byte nonce and authenticated ciphertext.
- Coordinator relay only forwards this route when the method is POST and the body is a valid uncompressed P-256 public key.
- Browser bundle and focused source assertions are kept aligned. Age identity support for metadata decryption and other features remains intact.

## Verification

Focused ECDH JavaScript assertions pass, browser source and generated bundle pass `node --check`, bundle regeneration completed, and `git diff --check` passes. The broad coordinator workspace test has six unrelated baseline failures where existing assertions expect UI/access patterns not present in GitHub `main`; the ECDH-specific assertion passes independently. Rust formatting and tests could not start because the local `rustc`/`rustfmt` runtime aborts while loading a missing `libz3.5.1.dylib`; GitHub CI must verify the Rust changes.

## Recovery

Until merge, the branch can be recreated from `origin/main` and the focused commit. The shared checkout and unrelated worktrees were not changed.
