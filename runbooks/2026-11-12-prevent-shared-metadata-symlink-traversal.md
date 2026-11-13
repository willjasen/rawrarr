# Prevent shared metadata symlink traversal

Issue: [#457](https://github.com/willjasen/rawrarr/issues/457)

The security scan flagged filesystem reads in artwork-cache, config-repair, and shared metadata paths. Artwork paths are derived from numeric provider IDs and opaque HMAC filenames, while configuration repair uses the configured shared-data root; neither accepts an arbitrary request path. However, synchronized entries can be attacker-influenced, and filesystem reads followed symlinks. The fix opens files with `O_NOFOLLOW`, verifies regular-file type, applies byte limits while reading, validates metadata subdirectories, and copies legacy ciphertext from the verified open file handle instead of reopening its path. Cached and uncached artwork/metadata loads share the same protections.

No persistent data, protocol, schema, or encrypted-file format changes. Existing deployments need no migration; valid regular files remain readable, while symlinked or oversized ciphertext is rejected.

## Verification

- Focused Rust filters `safe_fs`, `artwork_cache`, `config_repair`, `hmac_metadata_filename_tests`, `metadata_cache`, `config_schema_tests`, `filename_secret_tests`, and `metadata_storage_tests` passed in the isolated OrbStack test clone (31 tests).
- `git diff --check` passed.
- The full Rust, Node.js, and Playwright suites were not run.
