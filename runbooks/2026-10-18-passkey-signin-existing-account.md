# Simplify passkey sign-in for returning users

## Objective and initial state

Remove the recurring username prompt from passkey sign-in. PostgreSQL already stores the username/display name and determines whether first-user setup is complete, but the page showed a username field whenever an account existed.

## Actions

- Returning users now see only **Sign in with passkey**; the login start request can omit a username and the server selects the existing account from PostgreSQL, using its stored username for WebAuthn.
- First-time setup still checks PostgreSQL status, then asks only for a display name and creates the passkey. The server derives the internal username from that name; existing API callers may still provide an explicit username.
- Updated the passkey integration test, design guide, and user guide.

## Verification

- The isolated Docker test suite passed: 7 Node tests, the age setup Playwright test, and 10 Rust tests.
- `test_rawrarr_passkey.py` passed Python syntax compilation. Its full disposable-PostgreSQL WebAuthn flow was not run in the isolated test runner.
- The host Rust formatter remains unavailable because the installed LLVM toolchain depends on a missing Z3 library; the production Docker build compiled the application successfully.

## Final state and remaining work

The updated local web service is healthy on `127.0.0.1:7297`, and `/health` returned `ok`. Changes remain local and uncommitted alongside pre-existing worktree changes.
