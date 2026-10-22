# Age ID library visibility regression

## Objective

Ensure coordinator Movies and TV pages do not render library data when this browser has no age private ID available.

## Initial state

The shared checkout contained unrelated staged and unstaged changes, so work was isolated in `/Users/willjasen/.codex/worktrees/age-id-library-visibility/rawrarr`. Existing Movies unlock logic returns when no age ID is present. TV navigation rendered its plaintext browser cache regardless of age ID, exposing cached library details on the coordinator UI without unlocking.

GitHub issue search and creation were attempted for `willjasen/rawrarr` but the connected GitHub API returned HTTP 422 and 404 respectively. Proposed issue:

> Test that Movies and TV remain hidden without an age ID. Acceptance: test both pages with no age ID entered or loaded; assert decrypted library data does not render or appear in browser-visible content; keep coverage focused on the coordinator privacy boundary.

## Actions

- Gate cached Movies and TV library rendering and TV refresh loading on an age ID already loaded in browser memory.
- Add source-level and Playwright regression coverage using seeded private cache entries without an age ID.

## Verification

- Focused source regression passed: `node --test --test-name-pattern='Movies and TV cached library data stays hidden' scripts/tests/test_rawrarr_coordinator_workspace.mjs` (1 pass).
- Focused Chromium Playwright regression passed in the isolated `rawrarr-testing-base:local` container: `node --experimental-global-webcrypto --test --test-name-pattern='Movies and TV do not display cached library data without an age ID' scripts/tests/test_rawrarr_coordinator_sonarr_settings.mjs` (1 pass). The test seeds distinct Movies and TV cache entries, confirms the age ID is absent, and verifies neither page displays its fixture.
- `git diff --check` passed.
- The complete coordinator workspace test file was attempted earlier and had seven unrelated failures because this detached base commit predates the current coordinator workspace contracts; the new source assertion passed in that run.

## Recovery

Revert the age-ID condition on cached TV rendering and remove the matching regression assertion.
