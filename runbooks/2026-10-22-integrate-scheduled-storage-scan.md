# Integrate scheduled storage scan controls

## Objective and issue

Bring the scheduled storage scan work from `willjasen-observe-scheduled-storage-scan` onto current GitHub `main` through a focused pull request. Tracked by [issue #44](https://github.com/willjasen/rawrarr/issues/44).

## Initial state

The source branch had four unique commits and a clean worktree, but was 165 commits behind `origin/main`. The shared `main` checkout contained unrelated staged and unstaged work, so integration used a separate worktree based on current `origin/main` (`fc49caf`).

## Actions

- Cherry-picked the four scheduled-scan commits onto a clean PR branch and resolved overlaps while retaining current `main` content and terminology.
- The change adds persistent daily/weekly wall-clock schedules with IANA timezone handling, owner-facing schedule controls and status, storage-only routes, regression coverage, and associated docs/runbooks.
- Created a new task branch from current `origin/main`; the shared checkout was not edited.

## Verification

- `node --test scripts/tests/test_rawrarr_storage_scan_ui.mjs` passed (3 tests).
- `git diff --check origin/main...HEAD` passed.
- Local `cargo test storage_node` and `cargo fmt --check` could not start: the installed `rustc` aborts because its linked LLVM requires a missing Z3 library. PR CI is expected to provide Rust verification.

## Final state

The focused PR is pending GitHub checks and review. No deployment was performed. Recheck the Rust CI result and review the current `main` diff before merge.
