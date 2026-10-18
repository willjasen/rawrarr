# Verify private age key copy

## Objective

Require the user to demonstrate they can read the private age identity after copying it, before allowing vault unlock.

## Initial state

The copy button already copied only the `AGE-SECRET-KEY-PQ-1` private key line. Unlock was gated by a backup confirmation checkbox, but not by a check that the user could read the copied key. The key's leading format prefix is fixed, so fixed positions would not provide a meaningful check.

## Changes

- After a successful clipboard write, select three random alphanumeric character positions after the fixed key prefix.
- Keep the requested positions and response only in page memory; clear them when the identity changes, unlock succeeds, or the page closes.
- Require the correct characters and the backup confirmation before enabling **Unlock vault**. Apply the same flow after a pasted identity is copied.
- Update the design guide, technical status, user guide, and age setup test.

## Verification

- `node --test scripts/tests/test_rawrarr_age_setup.mjs` passed, including correct and incorrect challenge responses for both generated and pasted identities.
- `git diff --check` and Docker Compose config validation passed.
- Docker image rebuild succeeded; both `rawrarr` and `postgres` report `healthy` in Compose.
- `/health` returned `ok`. Pasted age-file comments are stripped from the masked input value before unlock; only the private key is submitted.

## Final state

The copy button places only the private key line on the clipboard. Vault unlock requires three random characters from after the fixed prefix plus the backup confirmation. Pasted files with comments are accepted even if the password input flattens line breaks.
