# Integrate shared Radarr recovery

## Objective and issue

Bring the shared Radarr recovery workflow from `willjasen-radarr-external-deployment` onto current GitHub `main` through a focused pull request. Tracked by [issue #46](https://github.com/willjasen/rawrarr/issues/46).

## Initial state

The source branch had four unique commits and a clean worktree, but was 165 commits behind `origin/main`. The shared `main` checkout contained unrelated staged and unstaged work, so integration used a separate worktree based on current `origin/main` (`fc49caf`).

## Actions

- Cherry-picked the four recovery commits onto a clean PR branch and resolved overlaps while preserving current `main` content.
- The change adds owner-authenticated Radarr backup downloads, shared encrypted recovery data, worker progress, a scoped restore utility, tests, and security/design documentation.
- Created a task branch from current `origin/main`; the shared checkout was not edited.

## Verification

- `python3 scripts/tests/test_rawrarr_radarr_recovery.py` passed (7 tests).
- `git diff origin/main...HEAD --check` passed.
- Rust and browser test suites were not run in this environment; check PR CI before merging.

## Final state

The focused PR is pending GitHub checks and review. No deployment was performed. Review the current `main` diff and CI results before merge.
