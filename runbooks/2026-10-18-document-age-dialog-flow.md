# Document the current age identity dialog flow

## Objective

Align the user guide, design guide, architecture, and roadmap with the implemented age identity copy and verification dialog.

## Initial state

Several documents still described random challenge positions or automatic unlock after character entry. The UI now uses fixed whole-key positions 24–26, explicit Continue/Unlock actions, and match feedback.

## Changes

- Updated `docs/user-guide.md` with the whole-key character positions, mismatch guidance, and explicit Continue action.
- Updated `docs/DESIGN.md` with the yellow recovery warning, prominent challenge panel, validation feedback, disabled-until-valid Continue button, no Close action, and explicit submission.
- Updated `docs/device-architecture.md` and `ROADMAP.md` with the current behavior and privacy boundary.

## Verification

Searched the authoritative docs for stale automatic-unlock and random-position wording; none remains in the current design description. `git diff --check` passed. No code or runtime state changed in this documentation task.

## Recovery

Revert only this task's documentation edits and this runbook index entry. The prior descriptions would no longer match the implemented dialog.
