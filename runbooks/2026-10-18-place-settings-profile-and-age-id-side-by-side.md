# Place Settings profile and age ID side by side

## Objective

Show the user profile and browser age private ID controls together in a shared row on wider screens, stacking them on narrow screens.

## Initial state

Settings rendered the profile and age ID storage cards as separate full-width blocks. The working tree contained extensive unrelated user changes; this task owns only the Settings page layout, this runbook, and its index entry.

## Actions

- Added a responsive identity grid wrapping the existing profile and age ID cards. The profile takes a narrower column, while the age ID card uses the remaining width. Below 900px, the cards stack.
- Preserved the existing card controls, IDs, and behavior.
- Removed the age ID browser-storage explanatory paragraph at the user's request.

## Verification

- `git diff --check` passed for the three task files.
- A focused source assertion confirmed the row wrapper and responsive two-column/one-column rules.
- Confirmed the requested paragraph is absent; `git diff --check` passed for the updated page and runbook.

## Final state

- The Settings profile and age private ID cards now share a responsive row, stacking below 900px. No deployment was performed.
