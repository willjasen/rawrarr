# Fix the age challenge start position

## Objective

Keep the three-character age identity challenge sequential and always start it at character 24 of the whole private key (character 5 after the fixed `AGE-SECRET-KEY-PQ-1` prefix).

## Initial state

The challenge start was randomly selected from the 4th through 7th suffix characters. In Chrome, the user saw a prompt at absolute private-key position 24 and requested that the start always be position 5 relative to the variable key text.

## Changes

- Replaced randomized selection with a fixed offset of four zero-based characters after the prefix (the 5th suffix character).
- Clarified both the whole-key position and position after the fixed prefix in the prompt. Updated the design guide, user guide, roadmap, and focused regression test.
- Rebuilt `rawrarr-local` from this checkout while preserving its existing runtime configuration and persistent mounts.

## Verification

Focused Docker test run passed: six Node.js tests and nine Rust unit tests. Fixture-dependent Python integration tests were skipped. The rebuilt `rawrarr-local` container is healthy, `/health` returns HTTP 200, and its binary contains the new two-position wording and no longer contains the old random-position prompt. Chrome page reload was not verified in this task because its tab was already claimed by another browser session; an open page can continue showing the previous challenge until refreshed.

## Recovery and remaining work

Revert only the task-owned source, test, documentation, runbook, and index changes to restore randomized start positions. Rebuild `rawrarr-local` from the previous source if rollback is needed. No private identity values or secret contents were read or changed.
