# Browser age-ID persistence

Tracking: [#179](https://github.com/willjasen/rawrarr/issues/179).

## Scope and decisions

Validated age private IDs already use origin-scoped browser `localStorage`.
Retain this behavior across reloads, navigation, and expired sessions, until
explicit clear, sign-out, or removal of site data. The temporary root signing
key remains memory-only. Browser storage is not an encrypted vault or a backup.
The subscriber's opt-in node-local copy is independent and is not deleted by
browser clear or sign-out. No private identity belongs in roarlay service data.

Correct only the stale technical-status workspace paragraph and related privacy
copy. Prevent pending validation/export/library requests from restoring an ID
or rendering unlocked libraries after clear/sign-out. Report storage-removal
failures rather than claiming success; retain the clear controls for retry and
do not navigate through sign-out while a saved key could remain. Handle another
tab's `localStorage.clear()` as well as removal of the individual key.

## Adoption and recovery

No persistent-data or protocol migration is needed: the storage key and format
are unchanged. Existing browser IDs restore automatically on the same origin.
Existing deployments adopt this via the normal UI release path; this task does
not deploy. Reload existing tabs after adopting the release. Storage policies
may prevent persistence; the workspace reports page-only availability. If
removal fails, allow storage access and retry, or remove this site's browser
data. Keep an independent private-ID backup before clearing site data.

Rollback changes UI code only; it does not rewrite synchronized or node-local
data. Browser copies remain until explicitly removed. Browser clear/sign-out
does not lock or erase an already unlocked subscriber's separate node-local ID.

## Verification

Focused Chromium coverage exercises save, real reload/navigation, another tab,
both clear controls, sign-out, storage failures, and delayed asynchronous work
at desktop 1280x800 and mobile 375x812. Age setup coverage confirms browser
persistence without opting into node-local persistence.

Ran `python3 scripts/tests/run_rawrarr_tests_in_docker.py` against the working
tree snapshot, using the existing reusable base and an offline test clone.
The final clone was `rawrarr-test-regression-tests-mw0ueflw`:

- Node.js: 156 passed.
- Rust: 123 passed.
- All eight new persistence browser cases passed, as did the extended age-setup
  case and all four existing cross-tab/restore cases.
- Full Playwright suite: one unrelated failure in
  `test_rawrarr_navigation.mjs:121`. Immediately after resizing the subscriber UI
  from mobile to desktop, `aria-hidden` was still `true` instead of `false`.
  This test and its `ui.html` implementation were not changed by this task; the
  same test passed in the prior runner invocation. The full browser suite is
  therefore not green, despite the passing age-ID coverage.
- `git diff --check` passed.

The browser tests use mocked API and cryptographic responses; they verify the
real UI/storage lifecycle, not end-to-end cryptography. No live deployment,
Python integration suite, peer-sync test, or full coordinator/two-node
environment test was performed.
