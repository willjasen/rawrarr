# Refresh linked regression test clones

Issue: [#208](https://github.com/willjasen/rawrarr/issues/208)

## Problem

The OrbStack Rust run passed 90 of 103 tests. Failures came from the minimal cached test image missing external tools/fixtures and dependencies introduced by the current copied source lockfile. The cached base was ARM64 but its `age` installation was hard-coded to AMD64, so age-dependent tests also lacked a usable native binary.

## Change

- Keep `rawrarr-testing-base:local` immutable and reusable for ordinary source changes.
- Create a unique linked test container, copy current Rust source, `scripts/tests`, and required repository fixtures, then prepare its writable layer with `prepare-test-clone.sh`.
- Install only missing test tools in the clone, select and checksum the age release for the clone architecture, validate required Rust fixture files, and run `cargo fetch --locked` against the copied `Cargo.lock`.
- Hold test execution behind `wait-for-test-trigger.sh`; after preparation, disconnect the clone from Docker's bridge and release the trigger so the suites run offline.
- Fix the base Dockerfile to install age for Docker's target architecture for future base rebuilds.

## Verification

- `node --test scripts/tests/test_rawrarr_docker_healthchecks.mjs` — passed, 4/4.
- `sh -n scripts/tests/prepare-test-clone.sh scripts/tests/wait-for-test-trigger.sh`, `python3 -m py_compile scripts/tests/run_rawrarr_tests_in_docker.py`, and `git diff --check` — passed.
- Rebuilt `rawrarr-testing-base:local` on native OrbStack `linux/aarch64` and ran `python3 scripts/tests/run_rawrarr_tests_in_docker.py --rebuild-base --remove-after`. Clone preparation reported `arch=arm64`, Rust 1.85.1, Node 18.20.4, and age v1.3.2. Rust passed **106/106**, including age-backed fixtures. The crypto/ENS browser regression group passed **12/12**.
- Overall runner exit was 1: Node passed 117/121 (the existing brand-case assertion and three Stripe tests whose `/workspace/stripe-service` fixtures are not copied into the clone failed); Playwright had 11 failing cases across existing artwork timing, movie links/responsiveness, navigation, shared-library, and Sonarr settings tests. Those unrelated suite issues remain outside this test-clone setup change. Python integration tests were skipped by the test runner because they require disposable PostgreSQL/age/Syncthing fixtures.

## Limits

No application deployment or production payment integration is part of this test-runner maintenance. The shared base should be rebuilt only when its pinned baseline toolchain or dependencies change; current source and lockfile refresh belongs in each new clone.
