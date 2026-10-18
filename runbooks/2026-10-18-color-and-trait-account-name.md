# Color and trait account name

## Objective and initial state

Add the dinosaur avatar's main color and a feature word between the generated account-name words. The worktree was already mixed with unrelated edits and untracked files; changes were limited to profile-name generation, its tests, this runbook, and the runbook/test indexes.

## Implementation

The browser derives color from the exact digest used to render the avatar. It labels the avatar `winged` or `wingless` from the same fourth digest byte used by the SVG's wing draw. Generated base-name pairs leave room for the longest color and feature labels within the 32-character account-name limit.

## Verification and deployment

- `node --test scripts/tests/test_rawrarr_profile_color.mjs scripts/tests/test_rawrarr_coordinator_workspace.mjs scripts/tests/test_rawrarr_passkey_ui.mjs` — passed (18 tests).
- Focused Rust `account_names_are_dinosaur_themed_and_unique_per_account_id` test in the offline Rawrarr test image — passed.
- `git diff --check` — passed.
- Rebuilt only `roarlay` in `rawrarr-local`; the container reports `healthy`.
- `https://rawrarr.risk-mermaid.ts.net/health` returned the Roarlay health page, and Serve read-back confirmed `:443` `/` proxies to `http://127.0.0.1:8080`.
- The roarlay rebuild marker was removed and confirmed absent.

## Recovery

Revert only the task-owned profile helper, UI integration, name-length filter, and associated test/runbook changes if this feature needs to be rolled back. Do not alter unrelated worktree changes.
