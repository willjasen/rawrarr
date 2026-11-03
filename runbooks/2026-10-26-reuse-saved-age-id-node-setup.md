# Reuse the saved age ID in node setup

## Objective and initial state

When an owner opens `/setup/node` to add a subscriber, reuse the age private ID already saved in that browser instead of requiring it to be pasted again. The coordinator uses the `rawrarr.browser.age-id` local-storage key, while node setup currently begins with an empty identity. Tracked by [issue #125](https://github.com/willjasen/rawrarr/issues/125).

Relevant earlier work: [remember the age ID after signup and first use](2026-10-25-remember-age-id-after-signup.md) and [simplify first-account onboarding](2026-10-22-simplify-first-account-onboarding.md).

## Actions

- Subscriber setup reads the established `rawrarr.browser.age-id` value, validates it locally, and uses it directly in tab memory when valid.
- Invalid or unavailable browser storage leaves the existing create/import controls available. The private ID is not put in an input or sent to roarlay.
- Restored the optional setup actions and explicit Compose-step toggle that were present in the embedded bundle but missing from its source module; rebuilt the bundle and added valid remembered-ID and invalid-ID fallback coverage.
- Updated the OrbStack test runner to copy the `.github` workflows fixture required by the existing workflow regression check.

## Verification

`python3 scripts/tests/run_rawrarr_tests_in_docker.py --context orbstack --remove-after` passed: Node.js 136/136, Playwright browser 1/1, and Rust 83/83. Python integration checks were skipped because the runner has no disposable PostgreSQL, age, or Syncthing fixtures. `git diff --check` passed. The browser test confirmed saved-ID reuse, invalid-ID fallback, and that private identity material stays out of the page and network requests.

## Recovery and remaining work

Revert the task-owned changes to restore manual age-ID entry as the only setup path. The saved ID remains browser-local and is cleared by the existing sign-out and remove controls.
