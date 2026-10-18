# Explain age challenge validation state

## Objective

When a user enters three characters but Continue remains disabled, show whether the answer is incomplete, incorrect, or verified, and remind them which whole-key positions to check.

## Initial state

A Chrome dialog showed three entered characters with Continue disabled but no message explaining the state. Automated UI coverage confirmed that Continue should enable only when the challenge response matches.

## Changes

- Added an accessible live status beneath the challenge field.
- Incorrect input now asks the user to recheck the displayed positions counted from the first character of the whole key; a correct answer confirms that Continue is ready.
- Normalize both sides of the comparison to uppercase while preserving the exact three-character check.
- Added regression checks for invalid and valid feedback and the disabled/enabled Continue behavior.
- Refreshed the local app at port 7297 so Chrome can use the updated UI.

## Verification

The Rawrarr test runner `rawrarr-testing-tests-mvditsiy` passed six Node.js tests and nine Rust unit tests. Rebuilt only the Rawrarr web service under `rawrarr-local` on `127.0.0.1:7297`; PostgreSQL remained healthy. The prior `/health` check returned `ok`. The current Chrome UI was unavailable after the test-only tab was closed, so the newest inline feedback has not been visually inspected in a live browser. The earlier refreshed dialog screenshot confirmed the yellow recovery warning, separated challenge panel, and disabled Continue action.

## Recovery

Revert task-owned changes to `rust-arr/src/age_setup.html`, `scripts/tests/test_rawrarr_age_setup.mjs`, this runbook, and its index entry. No identity content is stored by this change.
