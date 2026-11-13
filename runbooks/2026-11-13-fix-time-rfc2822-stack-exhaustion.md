# Fix Rust `time` RFC 2822 stack exhaustion advisory

Issue: [#467](https://github.com/willjasen/rawrarr/issues/467)

## Objective

Resolve [Dependabot alert #1](https://github.com/willjasen/rawrarr/security/dependabot/1) for `time` (`GHSA-r6v5-fh4h-64xc`, `CVE-2026-25727`). The alert identified `time` 0.3.45; upstream fixed the stack-exhaustion denial of service in 0.3.47 by limiting recursion depth while parsing RFC 2822 input.

## Changes and compatibility

- Updated `rust-arr/Cargo.lock` to `time` 0.3.47 and the required `time-core` 0.1.8, `time-macros` 0.2.27, and `num-conv` 0.2.2 releases.
- `cargo tree` confirms `time` 0.3.47 is used through the WebAuthn certificate-parsing dependency path.
- This dependency-only fix does not change persistent data, database schemas, encrypted configuration, or node/coordinator protocols. No migration is required. Existing deployments adopt the fix when rebuilt from the published application source/image.
- A rollback is a normal application-image rollback, but it restores the vulnerable dependency and should be treated as a security tradeoff.

## Verification

- Cargo updated the lockfile with `cargo update --package time --precise 0.3.47`; `cargo tree --locked --manifest-path /workspace/rust-arr/Cargo.toml -i time` showed `time` 0.3.47.
- `python3 scripts/tests/run_rawrarr_tests_in_docker.py --rebuild-base --remove-after` rebuilt the test image from its pinned Rust 1.88 Dockerfile. The Rust suite passed (141 passed, 1 ignored), and Playwright passed.
- The same run's Node.js suite had one unrelated failure: `left navigation renders every visible option in lowercase` expected the menu without `music`, while the current menu includes it. The other 152 Node.js tests passed.
- `git diff --check` passed.
- Host Rust commands were unavailable because the installed LLVM could not load `/usr/local/opt/z3/lib/libz3.5.1.dylib`; Cargo dependency resolution and the passing Rust suite ran in the isolated OrbStack test clone.
- Not deployed; deployment was not requested.
