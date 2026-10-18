# Resolve stale public age recipient during setup

## Objective and initial state

Continue reported a public-recipient save failure. Inspection found an existing public recipient from earlier setup, no config.aged or metadata, and 30 log files confirmed by magic bytes to be gzip rather than age ciphertext. The backend deliberately rejects a different recipient, but incorrectly presented the conflict as a storage error. See [temporary verification bypass](2026-10-18-disable-age-character-check.md).

## Actions

- Preserve the prior public recipient outside shared data, then remove it from the active shared folder only after confirming the folder contains no encrypted data or other files.
- Return HTTP 409 and explain that the original identity is required on recipient conflict. Preserve the prohibition on overwriting another identity’s recipient.
- Rebuild the local web service using existing configuration and persistent mounts on port 7297.

## Recovery

The prior public recipient is stored in Application Support/Rawrarr/recovery/age-recipient-before-2026-10-18-setup-fix. Restoring it requires ensuring no data has since been encrypted for the new identity. No private identity was read or stored by this task.

## Verification

Release build succeeded; GET /health returned `ok`; `git diff --check` passed. No automated tests or live browser unlock were run for this task. The user’s existing browser identity will be submitted when they retry Continue; no new private identity is needed for the repair.
