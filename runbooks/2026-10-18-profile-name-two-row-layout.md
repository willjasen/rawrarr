# Profile name two-row layout

## Objective and initial state

Display the four generated dinosaur profile-name words without separators in a two-row layout: color and trait on the first row, generated base-name words on the second. The repository worktree contains unrelated edits, which were left intact.

## Implementation

The profile heading now renders two text rows. The sidebar keeps the same four words joined without spaces or hyphens to fit its compact profile area. The avatar alternate text uses all four words.

## Verification and deployment

- `node --test scripts/tests/test_rawrarr_coordinator_workspace.mjs scripts/tests/test_rawrarr_profile_color.mjs scripts/tests/test_rawrarr_passkey_ui.mjs` — passed (18 tests).
- `git diff --check` — passed.
- The deployment attempt reached the Docker build but could not compile the current mixed Rust worktree (`cargo build --release --locked` reported 23 Rust type/method errors in unrelated files such as `secure_config.rs`). No container was recreated; the previous roarlay service remains running.
- The deployment lock was released and confirmed absent. The previously verified service health and tailnet route are unchanged, but this UI change is not deployed.

## Recovery

Revert only the profile-name rendering, focused assertion, this runbook, and index entry if the layout needs to be rolled back. Preserve unrelated worktree changes.
