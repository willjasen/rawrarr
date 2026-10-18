# Add an explicit Continue action to age setup

## Objective

After creating and copying an age ID, let the user enter the requested key characters and explicitly continue. Keep the action disabled until the entered characters match.

## Initial state

Generated identities hid the dialog's `Unlock vault` button and triggered unlock automatically as soon as the correct characters were typed. This left the user with only the Close action and did not provide a clear manual continuation step.

## Changes

- Show a `Continue` button when the generated-ID challenge appears.
- Keep it disabled until the ID has been copied, the backup confirmation is checked, and the requested characters match.
- Remove automatic unlock on input; clicking Continue performs the existing unlock flow.
- Extend the age setup UI test to cover visibility, disabled state, incorrect and correct answers, and explicit submission.

## Verification

The rawrarr-testing runner copied the current checkout into unique container `rawrarr-testing-tests-mvdinb6i`. All six Node.js tests and nine Rust unit tests passed. Fixture-based Python integration tests and OrbStack peer synchronization were skipped by the runner. A first run exposed a test path that still assumed automatic unlock; the test was updated to click Continue explicitly, and the full run then passed.

## Recovery

Revert only task-owned changes to `rust-arr/src/age_setup.html`, `scripts/tests/test_rawrarr_age_setup.mjs`, this runbook, and its index entry. No identity material or runtime state is involved.
