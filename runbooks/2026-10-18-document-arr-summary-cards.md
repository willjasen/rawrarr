# Document Arr summary cards

## Objective and initial state

Document the shared four-card renderer, Radarr and Sonarr-specific card wording, media-specific encrypted metadata subtotals, and TV backfill completion behavior.

## Changes

- Updated the product design, user guide, technical status, and documentation index.
- Added this task entry to the runbook index.

## Verification

- Focused summary-card and metadata-size tests passed in the preceding implementation turn.
- `git diff --check` passed.

## Final state

The documentation now describes the implemented behavior and how future Arr integrations should reuse the summary-card renderer. Changes remain uncommitted with existing worktree modifications.
