# Temporarily disable age character verification

## Objective and initial state

The user reported repeated rejection of copied-key characters and requested disabling the check until later. Copy acknowledgement remains required. See [prior feedback work](2026-10-18-age-challenge-validation-feedback.md). The working tree already contained extensive unrelated and earlier task changes.

## Actions

- Removed character challenge UI and validation. Successful clipboard copy plus the saved-ID checkbox enables Continue for generated IDs or Unlock vault for pasted IDs.
- Kept the yellow recovery warning and explicit forward action. Progress and unlock errors now appear inside the dialog.
- Updated focused regression coverage and current behavior documentation; recorded restoring verification in ROADMAP.
- Rebuilt only the rawrarr-local web service using its existing mounts/environment and port 7297. No data or identity files were changed. Chrome windows were left untouched.

## Verification

- Focused Node test passed 1/1 in `rawrarr-testing-tests-mvdj51v4`, created from the existing reusable testing base with networking disabled. Checked copy/acknowledgement gates, explicit submission, reset behavior, and pasted private-key-only handling.
- Local release image built successfully. GET http://localhost:7297/health returned `ok`; web and PostgreSQL reported healthy.
- Full suite and live Chrome flow were not run. A pre-existing browser challenge test still targets the old behavior and needs coordination with its ongoing work.

## Recovery and remaining work

Restore only this task’s source changes and rebuild the web service to restore the old check. Runtime data is unchanged. Reintroduce verification only after the reported failure is resolved and exercised in a real browser. Refresh an already-open setup page to load the revised UI, saving any newly created identity first.
