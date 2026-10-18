# Make the age identity challenge input visible

## Objective

Make the three-character identity verification field easy to see and ready to use after the user confirms that they copied and saved the private identity.

## Initial state

The input was present in the copy dialog, but the page only styled password inputs. The challenge field was plain `type="text"` and did not receive focus when revealed. Existing unrelated changes in `docs/node-types.md`, `runbooks/README.md`, and `runbooks/2026-10-18-rawrarr-container-distribution.md` were preserved.

## Changes

- Added explicit dark-theme styling, readable monospaced text, and a visible focus ring to the challenge input.
- Focused the input when the checkbox reveals the challenge.
- Extended the age setup test to verify the styling and focus behavior; updated the test index.

## Verification

Focused Docker test run passed: six Node.js tests and nine Rust unit tests. The runner skipped fixture-dependent Python integration tests and separately scoped OrbStack peer-container sync.

## Recovery and remaining work

Revert only the task-owned page, test, index, and runbook changes to restore the previous field presentation. No identity data or runtime state was changed.
