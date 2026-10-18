# Coordinator browser tab name

## Objective

Show `rawrarr` in the browser tab for the coordinator workspace instead of `Rawrarr workspace`.

## Initial state

The workspace document title was set in `rust-arr/src/coordinator_workspace.html`. The repository already had extensive unrelated staged-state candidates and untracked files; changes for this task are limited to the title, its focused regression assertion, and this runbook/index entry.

## Actions

- Changed the coordinator workspace `<title>` to `rawrarr`.
- Added an assertion to the coordinator workspace test to guard the document title.

## Verification

- `node --test scripts/tests/test_rawrarr_coordinator_workspace.mjs` passed (6 tests).
- Deployment was not performed: this checkout's `rawrarr-local` Compose project has no app service bound to `127.0.0.1:7297`; the coordinator container is published on `127.0.0.1:7298`, and the required `rawrarr-local` app/health endpoint on 7297 is unavailable. The Tailscale Serve route still targets the coordinator on port 8080.

## Final state

The coordinator workspace specifies `rawrarr` as its document title. The source change and regression assertion are ready; runtime deployment and live tab verification remain pending because the documented local app service on port 7297 is not running in this checkout's Compose project.
