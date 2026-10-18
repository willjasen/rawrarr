# Clarify age identity setup actions

## Objective and initial state

The user created an age identity but could not distinguish the file-selection action from the vault-unlock action. In the UI, “Import from file” loaded a saved identity into the masked field without unlocking; “Import and unlock” submitted the identity to unlock the encrypted configuration. The wording made these separate steps unclear, especially for a newly generated identity that was already held in page memory.

## Changes

- Rename the file action to **Load identity from file** and the submit action to **Unlock vault**.
- Explain that loading a file only fills the masked field, while a generated, pasted, or loaded identity is used when the user selects **Unlock vault**.
- Update the user guide and living design guide with this distinction.
- Extend the age setup test to verify the labels and that file selection does not trigger unlock.

## Verification

Run `node --test scripts/tests/test_rawrarr_age_setup.mjs` and `git diff --check`. Rebuild Rawrarr in OrbStack at port 8084 without reloading the user's current page, which may hold a newly created identity in memory. The test passed (1/1), the Docker rebuild completed, the Rawrarr container is up on `127.0.0.1:8084`, and `/health` returned `ok`. Leave the generated-identity tab untouched so the user can copy and unlock it.
