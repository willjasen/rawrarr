# Fix the OrbStack test runner copy step

## Objective

Resolve issue #55's test-runner `FileExistsError` so isolated OrbStack regression runs can start from the current checkout.

## Environment and starting state

- Worktree: `codex/test-failure-fixes` at `/Users/willjasen/.codex/worktrees/rawrarr-test-failures`.
- The runner copied the complete `rust-arr` tree into a temporary staging directory, then copied `web-export` a second time to the same staging destination. That duplicate `shutil.copytree` raised `FileExistsError` before the container started.
- The base image is dependency-only. The per-run container receives the current Rust source and tests, plus root fixtures required by source-structure tests.

## Changes

- Removed the duplicate UI and `web-export` copy steps. The current UI assets are already part of the copied Rust source tree.
- Kept copying all root Compose files and the selected `admin/`, `docs/`, and Caddy fixtures needed by source-structure tests.
- Added regression coverage that prevents reintroducing a second `web-export` tree copy.

## Verification

- Focused Node checks: `node --experimental-global-webcrypto --test scripts/tests/test_rawrarr_docker_healthchecks.mjs scripts/tests/test_rawrarr_age_setup.mjs` — 3 passed.
- Python syntax check: `python3 -m py_compile scripts/tests/run_rawrarr_tests_in_docker.py` — passed.
- OrbStack full runner started without `FileExistsError`: `python3 scripts/tests/run_rawrarr_tests_in_docker.py`.
- Full result: Playwright passed; Rust passed (79 tests); Node had 85 passing and 32 failing tests. The container retained logs as `rawrarr-testing-tests-mvk8lvum`.

## Final state and remaining work

The duplicate directory creation is fixed. The complete suite is not green because 32 Node regression cases still fail against the current local snapshot; resolve those separately before treating the repository as fully verified. The original shared checkout was not edited by this task. No deployment was performed.
