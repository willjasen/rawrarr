# Test age setup in Chromium

## Objective

Verify age identity creation and backup acknowledgement through an actual browser, and run that browser test in the isolated Rawrarr testing container.

## Initial state

The age setup flow had Node-based DOM simulation coverage but no real browser coverage. The test image included Node.js but not a browser or Playwright. During this task, existing uncommitted UI changes removed the three-character challenge, so the browser test covers the current generated-ID and backup-acknowledgement flow without changing that UI work.

## Changes

- Added a headless Chromium test covering generated-ID dialog, private-key display protection, clipboard copy, backup acknowledgement gating, and unlock submission.
- Added system Chromium and pinned `playwright-core` to the reusable testing image. The browser test uses a local fixture server and makes no external network requests.
- Indexed the test and described the updated image in the test documentation.
- Renamed the idle base-image container to `base-test-image-mvdgto87` within the `rawrarr-testing` project.

## Verification

The isolated `rawrarr-testing` run passed all 7 Node.js tests, including headless Chromium startup and the generated-ID backup flow, and all 9 Rust unit tests. The existing Python integration tests remained skipped by the container runner as documented. The test fixture serves the setup page and its API responses locally, so the test works with the runner's disabled network.

An initial run timed out after the success response because the fixture did not serve the post-unlock root-page navigation. The fixture now serves the setup page at both its entry path and root, and the browser test verifies the redirect and API submission.

## Recovery

Remove the task-owned browser test, its test index entry, the Chromium/Playwright Dockerfile installation, and this runbook/index entry. The persistent `rawrarr-local` service and its data are not part of this task.
