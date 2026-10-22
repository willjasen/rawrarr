# Skip GitHub Actions for Markdown-only changes

## Objective

Avoid starting the test workflow when a push or pull request changes only Markdown files, while retaining manual dispatch.

## Environment and initial state

- Repository: `willjasen/rawrarr`; isolated branch `codex/skip-md-only-actions` based on GitHub `main` at `f0a85df`.
- The original shared checkout contained unrelated staged and unstaged edits; this change was transferred into an isolated worktree to keep them untouched.
- GitHub issue search and creation returned API errors (422 and 404) through the connected account.

## Proposed issue

**Skip GitHub Actions for Markdown-only changes**

Acceptance criteria: (1) pushes and pull requests containing only `*.md` changes do not start the test workflow; (2) any non-Markdown change still starts it; (3) manual `workflow_dispatch` remains available.

## Change

Added `paths-ignore: ['**.md']` to the push and pull-request triggers in `.github/workflows/tests.yml`. Manual dispatch is unchanged. GitHub path filters evaluate the full changed-file set, so a mixed documentation and code change still starts the workflow.

## Verification

- Reviewed the isolated branch diff to confirm only the workflow, this runbook, and the runbook index changed.
- YAML parser and `actionlint` were unavailable; GitHub Actions behavior has not yet been exercised.
- No tests were run because this change only affects workflow triggers.

## Final state

The branch is ready for review and merge. Issue creation was unavailable via the connected GitHub account.
