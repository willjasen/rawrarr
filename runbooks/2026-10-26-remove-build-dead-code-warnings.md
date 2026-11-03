# Remove build dead-code warnings

Issue: [#191](https://github.com/willjasen/rawrarr/issues/191)

Removed the unused `AgeVault::read_shared_recipient` helper and the obsolete UI bundle signature status type and reader. The latter became unused after UI signature enforcement and its badge were removed. These definitions had no callers, so removing them does not change runtime behavior.

## Verification

- `git diff --check` passed.
- Confirmed neither warning symbol nor `SignatureStatus` remains in Rust source.
- `cargo check --release --locked` could not run because this worktree environment has no `cargo` or `rustup` executable.
- No GitHub Actions build was triggered, as requested.
