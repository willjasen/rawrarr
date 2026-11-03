# Dashboard SSE regression tests

Issue: [#147](https://github.com/willjasen/rawrarr/issues/147)

## Objective and initial state

Add regression coverage for the dashboard's `/api/events` server-sent events route and browser `EventSource` connection. The latest fetched `origin/main` had a working route and browser handler but no focused tests for either. Work is on a dedicated branch created from that ref.

## Changes

- Added a Rust handler test that reads the first emitted SSE frame and checks its content type, cache header, named event, JSON shape, and fixture device state.
- Added a Chromium test that controls the browser's `EventSource`, then checks the `/api/events` URL, snapshot rendering, and connected/reconnecting labels.
- Added the test to `scripts/tests/README.md`.

## Verification

- Focused Chromium test passed in the isolated OrbStack dependency image: 1/1.
- Focused Rust handler test passed in the isolated OrbStack dependency image: 1/1.
- The full test runner passed all 110 Node tests, but its initial browser run exposed a wrong element ID in this new test and its Rust compile exposed a missing import. Both were fixed, and each focused test then passed. The full suite was not rerun after those fixes.
- Direct host Rust execution is unavailable because the installed LLVM runtime cannot load its Z3 dylib. The isolated runner's Rust toolchain works.
- `node --check` and `git diff --check` passed.

## Final state

The new focused checks pass. No application behavior changed. Changes are ready for review on the task branch; no commit or push was requested.
