# Remember the age ID after refresh

## Objective and initial state

Fix the setup flow so selecting **Keep this ID on this device** persists the age identity for later app starts. The browser posts `rememberOnDevice`, but the Rust request type used `remember_on_device` without camel-case deserialization, so Serde silently used its default `false`. The existing [local age ID runbook](2026-10-18-remember-age-id-local.md) describes the intended node-local persistence behavior.

## Actions

- Enabled camel-case deserialization for the age identity request.
- Added a Rust regression test that decodes the browser-shaped request and checks the remember flag.

## Verification

- `python3 scripts/tests/run_rawrarr_tests_in_docker.py --remove-after` passed all 9 Node.js tests, the Chromium browser test, and all 12 Rust unit tests, including the new request-decoding regression. Python integration tests were skipped by the test runner because they need disposable service fixtures.
- `git diff --check` passed.
- Direct host `cargo test` could not start because the installed LLVM requires a missing Z3 dynamic library; the isolated project test container supplied the working Rust toolchain.

## Final state and remaining work

Selecting **Keep this ID on this device** now deserializes correctly and reaches the existing local persistence path. The runbook index was updated. No change was made to the persistent web service because this is a request-decoding fix with no page change. Existing uncommitted project work remains untouched.
