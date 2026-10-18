# Sequential age identity challenge

## Objective

Ask users to verify their copied age identity by entering three consecutive characters. Randomly choose the starting character from positions 4–7 after the fixed `AGE-SECRET-KEY-PQ-1` prefix.

## Initial state

The setup page requested three unrelated random character positions from the private identity suffix. The age setup test checked that the positions skipped the fixed prefix. Existing unrelated work was present in `docs/node-types.md`, `runbooks/README.md`, and `runbooks/2026-10-18-rawrarr-container-distribution.md`; it was preserved.

## Changes

- Changed challenge generation to pick one random suffix offset from 3 through 6 and use that character plus the next two.
- Changed the prompt to state the one-based starting position and that three consecutive characters are required.
- Updated the age setup test to check the position range, sequence wording, and successful response handling.
- Updated design, user, and roadmap documentation to describe the current behavior.

## Verification

Focused Docker test run passed: six Node.js tests and nine Rust unit tests. The runner reported Python integration tests skipped because they require disposable PostgreSQL, age, or Syncthing fixtures; OrbStack peer-container sync is also a separate host-context test.

## Recovery and remaining work

Revert only the task-owned source, test, documentation, and this runbook changes to restore the previous challenge behavior. No runtime data or secrets were changed.
