# Keep the coordinator workspace pending while it reconnects

## Objective and issue

When the coordinator becomes unavailable during an update or restart, keep an already-open workspace in a clear reconnecting state and resume it when roarlay returns. GitHub issue [#98](https://github.com/willjasen/rawrarr/issues/98) tracks the behavior.

Related procedures: [coordinator health page](2026-10-18-coordinator-health-page.md) and [preserve the page on refresh](2026-10-18-preserve-page-on-refresh.md).

## Initial state

- The signed-in workspace polls nodes and pending approvals every 10 seconds, but did not have a coordinator-wide availability view.
- The task uses a dedicated worktree based on fetched `origin/main` commit `7b9d888`.

## Implementation

- Added a same-origin `/health` monitor that checks every three seconds, times out slow checks, requires two failed checks before marking the coordinator unavailable, and waits for two successful checks before recovery.
- Added an accessible modal pending view with a reconnect message and activity indicator. A single transient request failure does not obscure the workspace.
- On recovery, close the pending view, refresh node and approval data, and re-run the current view's loaders without navigating away or reloading the document.
- Served the monitor as a coordinator asset and added it to the signed UI bundle allowlist.
- Added focused Node regression tests and this runbook entry.

## Verification

- `python3 scripts/tests/run_rawrarr_tests_in_docker.py --remove-after` passed: 133 Node tests, the Playwright browser test, and 82 Rust tests. The optional Python integration tests were skipped by the repository runner because their disposable PostgreSQL/age/Syncthing fixtures are not part of that unit-test container.
- `node --check` passed for the coordinator workspace and both modified browser test modules.
- `python3 scripts/tests/test_ui_bundle.py -v` passed all five UI bundle packaging checks.
- `git diff --check` passed.
- A host `cargo check --locked --manifest-path rust-arr/Cargo.toml` could not start because the host's rustc/LLVM cannot load its configured Z3 dylib (`/usr/local/opt/z3/lib/libz3.5.1.dylib`); the isolated Rust suite compiled the application tests and passed.
- No live browser deployment is included. Live coordinator update/restart behavior remains to be verified after publishing and deploying from GitHub `main`.
- `node --check` passed for the coordinator workspace and both modified browser test modules.
- `python3 scripts/tests/test_ui_bundle.py -v` passed all five UI bundle packaging checks.
- `git diff --check` passed.
- A host `cargo check --locked --manifest-path rust-arr/Cargo.toml` could not start because the host's rustc/LLVM cannot load its configured Z3 dylib (`/usr/local/opt/z3/lib/libz3.5.1.dylib`); the isolated Rust suite compiled the application tests and passed.
- No live browser deployment is included. Live coordinator update/restart behavior remains to be verified after publishing and deploying from GitHub `main`.

## Recovery and remaining work

If the reconnect dialog regresses, revert the coordinator availability module, its page integration, route, bundle allowlist entry, and focused test. The page can only show this state after its workspace code has loaded; a browser opened for the first time while the coordinator is fully offline still gets the browser's normal connection error.
