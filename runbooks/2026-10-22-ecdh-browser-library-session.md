# Use ECDH for encrypted browser library sessions

## Objective and initial state

Integrate issue [#35](https://github.com/willjasen/rawrarr/issues/35): replace the age-wrapped session-key exchange used for the encrypted Radarr browser library response with an ephemeral P-256 ECDH exchange and HKDF-derived AES-GCM key. Keep age identities and age-encrypted metadata for the other features that use them.

Work is isolated in `codex/ecdh-browser-library-session`, based on GitHub `main` at `6814b3b`. The shared `main` checkout had staged, unstaged, and untracked work and was 31 commits behind GitHub; it was left untouched. The browser source and Rust node/coordinator still used the older age-wrapped response handshake on GitHub `main`.

## Changes

- Browser creates an ephemeral P-256 key pair, posts the 65-byte uncompressed public key, derives the shared secret from the node's ephemeral public key, and uses HKDF-SHA-256 with fixed protocol salt/info to derive an AES-256-GCM response key.
- Metadata node validates the browser public key, creates its own ephemeral P-256 key, derives the same key, and returns its public key followed by a random 12-byte nonce and authenticated ciphertext.
- Coordinator relay only forwards this route when the method is POST and the body is a valid uncompressed P-256 public key.
- Browser bundle and focused source assertions are kept aligned. Age identity support for metadata decryption and other features remains intact.

## Verification and merge result

- Focused ECDH JavaScript assertions pass; browser source and generated bundle pass `node --check`; bundle regeneration and `git diff --check` pass.
- GitHub PR [#36](https://github.com/willjasen/rawrarr/pull/36) merged into `main` on 2026-10-22 as merge commit `97481c52241038f57211fa8778dfd565137b4279`. `origin/main` was fetched and verified at this commit. GitHub's `Basic script tests` check passed; this workflow runs standard-library Python tests, not Rust.
- The isolated test runner's Playwright browser test passed. Its aggregate Node.js suite reported 69 passing and 23 failing tests, including fixture/UI assertions that do not match the current repository snapshot. The focused ECDH assertion passed independently.
- Rust tests did not compile: offline Cargo could not find cached crate `mime_guess`, and the test container has no network. Local `cargo fmt`/`cargo test` also could not start because the host Rust tools abort while loading missing `libz3.5.1.dylib`.
- Full project verification remains incomplete. Resolve the Rust test environment and investigate the unrelated Node.js suite failures separately.

## Recovery

The code change is merged; use GitHub `main` at `97481c5` as source of truth. The ECDH task worktree and local branch were removed after merge. The shared checkout and unrelated worktrees were not changed by this task.
