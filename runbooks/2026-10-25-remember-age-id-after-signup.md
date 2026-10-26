# Remember the age ID after signup and first use

## Objective

Save a newly created age ID in browser-local storage after vault unlock, restore it on the main workspace, save IDs after successful first validation, and clear the saved value on sign-out or manual clear. Track [issue #70](https://github.com/willjasen/rawrarr/issues/70).

## Initial state

- Worktree branch: `codex/remember-age-id-signup`, based on `origin/main` at `591b2c4`.
- The age setup page only passed the optional `rememberOnDevice` setting to the node.
- The coordinator workspace deleted `rawrarr.browser.age-id` on load and otherwise kept IDs in tab memory.
- Relevant prior context: [remember age ID in browser](2026-10-18-remember-age-id-in-browser.md).

## Actions

- After successful generated-ID or imported-ID vault unlock, write the private ID to the browser's local storage.
- Restore the value into the coordinator workspace and save IDs after successful metadata-node validation or metadata export.
- Clear the browser value on manual clear and on sign-out from both workspace entry points.
- Update regression coverage and this runbook as implementation and verification progress.

## Verification

- `node --test scripts/tests/test_rawrarr_age_setup.mjs scripts/tests/test_rawrarr_coordinator_workspace.mjs` — 26 tests passed.
- `python3 scripts/tests/run_rawrarr_tests_in_docker.py --remove-after` — Node.js 122 passed, Playwright browser 1 passed, Rust 82 passed. Python integration checks were skipped by the test runner because its disposable PostgreSQL, age, or Syncthing fixtures are not part of that unit-test container.
- `git diff --check` — passed.
- No live deployment was requested or performed.

## Final state

- Signup vault unlock and successful first age-ID validation now save the ID in browser-local storage. The coordinator workspace restores it on reload; manual clear and sign-out remove it.
- The subscriber UI sign-out handler also removes the browser-local and legacy tab-session values.
- Feature changes are committed on `codex/remember-age-id-signup` as `22a74da36ac4944090fe9e3918ec93af7891aa3a` (`feat: remember age ID in browser`). The branch was rebased onto current `origin/main` at `a273a0b` while preserving the intervening runbook-index entry.
