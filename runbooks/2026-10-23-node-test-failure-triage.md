# Node test failure triage and verified test save

## Objective

Diagnose the Node failures from issue #55's local OrbStack run, fix genuine errors where they can be separated from the current uncommitted policy/UI snapshot, and save a verified test change.

## Starting state

- Worktree: `codex/test-failure-fixes` at `/Users/willjasen/.codex/worktrees/rawrarr-test-failures`.
- The OrbStack test runner fix is committed as `1067ee2`.
- The checkout contains broad uncommitted source and test changes, including conflict markers in `docs/device-signing-design.md`, `runbooks/README.md`, and `scripts/tests/README.md`.
- Latest complete test container `rawrarr-testing-tests-mvk8lvum` reported 85 Node tests passing and 32 failing; Playwright passed; Rust passed 79 tests.

## Findings and actions

- The Node failures are predominantly source-structure assertions or browser assertions that disagree with current uncommitted changes. Examples include changed direct-route behavior, removed profile/device signature and avatar UI, and changed age-gating behavior. These are not attributable to the runner's duplicate-copy error.
- A focused failure also occurs in the Sonarr browser-settings fixture; the fixture retained route stubs for modules removed by the current uncommitted implementation. The worktree already removes those now-unused stubs, but the browser flow still times out, so this behavior is not recorded as fixed.
- Reviewed the changed age-setup test and ran it independently in the OrbStack test image.

## Verification

- `node --experimental-global-webcrypto --test scripts/tests/test_rawrarr_age_setup.mjs` in `rawrarr-testing-tests-mvk8lvum` — passed (1/1).
- Full suite from the retained test container — Node 85 passed / 32 failed; Playwright passed; Rust 79 passed.
- No source fix is claimed for the remaining Node failures because they stem from behavior changes in the uncommitted snapshot and require a policy decision about which behavior is intended.

## Final state and remaining work

The verified age-setup test adjustment is saved on this branch. Remaining Node failures need the intended behavior confirmed and the matching tests or implementation reconciled. The full suite is not green. The test container remains a local diagnostic artifact.
