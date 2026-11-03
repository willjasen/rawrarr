# Lowercase sidebar options

## Request and acceptance criteria

Render every option in the coordinator's left navigation in lowercase, matching the supplied screenshot. Navigation routes, page headings, and accessible names keep their existing text.

GitHub issue [#252](https://github.com/willjasen/rawrarr/issues/252) tracks this change and its acceptance criteria.

## Implementation

Added lowercase text transformation to `.nav-text` and regression coverage for all eight labels in `test_rawrarr_coordinator_workspace.mjs`. This only changes presentation; it has no persistent-data, schema, or protocol impact and requires no migration.

## Verification

Pending: run `node --test scripts/tests/test_rawrarr_coordinator_workspace.mjs` and `git diff --check`.

## Rollback

Remove the `.nav-text` lowercase transformation and its corresponding regression assertion.
