# Copy only the private age identity

## Objective

Ensure the setup page's **Copy private identity** button copies the secret identity line without the generated file's public-recipient comment.

## Initial state

The button copied the complete text returned by `age-keygen`, including `# created` and `# public key` comment lines. The unlock flow accepts the full text, so its parsing and submitted value should remain unchanged.

## Changes

- Added a browser helper to extract the `AGE-SECRET-KEY-PQ-1` line for clipboard use.
- Added regression coverage for both a newly generated identity and a pasted age file containing a public-recipient comment.

## Verification

- `node --test scripts/tests/test_rawrarr_age_setup.mjs` passed, including generated and pasted identities with public-recipient headers.
- `git diff --check` passed.
- Docker Compose config validation and `docker compose ... up -d --build rawrarr` succeeded.
- `http://localhost:8084/health` returned `ok`; the rebuilt container is running and retains the Finder-accessible `/shared` mount.

## Final state

The copy button places only the private key line on the clipboard. The full identity text remains in the masked input for unlocking, where age headers are still accepted.
