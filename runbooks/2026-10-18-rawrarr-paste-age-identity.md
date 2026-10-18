# Paste an existing age identity during setup

## Objective and initial state

Remove **Load identity from file** from the age setup flow and let users paste an existing private identity into the masked field. The setup page had a file input and button that populated the same password field; its separate vault-unlock and backup-confirmation steps already worked.

## Changes

- Removed the file input, file-selection button, associated browser handler, and file-specific instructions.
- Labeled the masked field for pasting an existing private identity. Pasting resets the backup acknowledgement and does not submit the identity; the user must still confirm their backup and choose **Unlock vault**.
- Updated the product design guide and user guide.
- Updated the browser test to cover paste and verify the file-selection UI is absent.

## Verification

The focused Node test passed (1/1), and `git diff --check` passed. The Docker image built successfully and the existing app container restarted under project `rawrarr-local` with its original shared-data path, password file, and named volumes. `/health` returned `ok`. An unauthenticated request to `/setup/age` returned `401 Unauthorized`, so served HTML could not be inspected without a logged-in browser session; the built image includes the tested updated source. No database or shared data was migrated.
