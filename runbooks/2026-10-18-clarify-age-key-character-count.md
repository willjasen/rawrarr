# Clarify age key challenge character counting

## Objective

Help users who receive a mismatch understand how to count the requested characters in a generated post-quantum age identity.

## Initial state

The user entered three characters but received a mismatch. The challenge implementation selects zero-based indexes 23–25, which correspond to whole private-key line positions 24–26. A test-generated `age-keygen -pq` identity confirmed this offset and produced three supported characters at those positions. The exact private key and user-entered response were not retained.

## Changes

- Reworded the challenge to say positions 24–26 of the private-key line, count from the first `A`, include all hyphens and digits, and ignore `#` comment lines.
- Updated the user guide and design guide with the same counting instructions.
- Updated the browser regression test to verify the prompt and position range.

## Verification

The `rawrarr-testing-tests-mvdiyl62` container passed six Node.js tests and nine Rust unit tests. A disposable `age-keygen -pq` fixture confirmed the generated-key parser's 24–26 offset without printing any key characters. Fixture-dependent Python integrations and OrbStack peer synchronization were skipped. Rebuilt only the Rawrarr web service under `rawrarr-local` on `127.0.0.1:7297`; its health check is healthy and `/health` returns `ok`. No Chrome tabs or windows were manipulated.

## Recovery

Revert only the task-owned prompt, test, documentation, this runbook, and index entry. No real identity or entered characters are recorded in this runbook.
