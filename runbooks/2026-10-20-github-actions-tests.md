# GitHub Actions test workflow

## Goal

Run a lightweight set of repository script tests automatically on pull requests and changes to `main`.

## Implementation

- Added `.github/workflows/tests.yml` for pull requests, pushes to `main`, and manual dispatch.
- The workflow uses Python 3.12 and runs standard-library unit tests discovered from `scripts/tests/test_*.py`.
- The Actions summary uses a colored status and compact pass/fail totals with Python and runner versions, elapsed time, and command; it lists failed test names only when needed. Verbose test output remains in the live job log.
- This deliberately omits Playwright, Rust, and integration suites for a fast, dependency-light initial CI check.
- The workflow requests read-only repository contents and does not require secrets.
- Fixed additional issues exposed by the local full-suite attempt: the test image now installs the locked web-export npm dependencies and avatar conversion tools; Node 18 test runs enable global Web Crypto; browser-oriented unit fixtures provide the globals and DOM elements required by the page scripts.
- Tightened configured direct-route host validation to Rawrarr tailnet names and corrected quality-profile metadata merging so cached fields survive while changed profile IDs update.
- Standardized the visible brand spelling as lowercase in the affected pages.

## Verification

`python3 -m unittest discover -s scripts/tests -p 'test_*.py' -v` passed locally (17 tests). The full isolated Rust suite passed (62 tests), and the affected focused Node tests passed (8 tests). The broader Node/browser aggregate still has unrelated existing failures in static source assertions and browser flows; it is intentionally not part of this minimal CI workflow. The hosted workflow still needs a GitHub Actions run to verify its runner environment.

## Files

- `.github/workflows/tests.yml`
- `TESTING.md`
- `runbooks/README.md`
