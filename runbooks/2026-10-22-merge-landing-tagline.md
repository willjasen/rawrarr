# Merge the Rawrarr landing tagline

## Objective

Integrate the existing `all the arrs` tagline enhancement into the current `main` landing page without importing the source branch's obsolete coordinator migration commit.

## Initial state

- `main` tracked `origin/main` at `5c5a86d` and had unrelated local changes in the coordinator, a coordinator browser test, and a runbook.
- A local tagline branch contained `ec06987` for the tagline and an earlier migration packaging commit. Current `main` already had newer migration numbering, so the migration packaging change was excluded.
- Existing `rawrarr` landing page and the signed-out landing page test were already on `main`.

## Actions

- Cherry-picked only the tagline commit's content, resolving conflicts by preserving the current main landing-page content and all newer runbook entries.
- Added the tagline under the product name and added a browser assertion for its text and vertical placement.
- Updated the design guide, landing test index, and runbook index.
- Reviewed the staged diff and confirmed it contains only the tagline implementation and related documentation/test changes.

## Verification

- `git diff --cached --check` passed before commit.
- `docker run --rm -v "$PWD:/workspace:ro" --entrypoint node rawrarr-testing-base:local --test /workspace/scripts/tests/test_rawrarr_landing_page.mjs` passed (1 test).

## Final state

- Tagline change committed to `main` as `9b1a93d` (`Add Rawrarr landing tagline`).
- The source branch's stale migration commit was not included.
- Existing unrelated worktree edits remain uncommitted and untouched.
- No deployment was performed as part of this Git integration.
