# Run Rawrarr tests in a uniquely named Docker container

## Objective and initial state

Run Rawrarr's self-contained Node and Rust tests inside the isolated `rawrarr-testing` Docker project, with results available in the test container's logs. Put a Unix timestamp in milliseconds encoded in lowercase base36 in testing container names and visible service labels; use the same scheme for the optional Syncthing web/storage peers. Keep the persistent `rawrarr-local` app and PostgreSQL separate. Build a reusable image base that behaves like a VM linked clone: common immutable image layers are shared, and each test run gets a separate copy-on-write writable layer.

## Changes

- Added `scripts/tests/Dockerfile` and `run-tests-in-container.sh` for Node tests and locked Rust unit tests. The base image includes Node.js and upstream age 1.3.2 binaries checked against the published SHA-256, fetches locked crates in a stable Docker build layer, copies current test sources, and precompiles Rust tests.
- Added `scripts/tests/run_rawrarr_tests_in_docker.py` to build or reuse `rawrarr-testing-base:local`, create a fresh uniquely named container with no network and no host source mount, label it as part of `rawrarr-testing`, follow its logs, and retain the stopped container by default. Docker's shared image layers provide the linked-clone-like base; each run gets its own writable layer. `--remove-after` removes it after displaying results.
- Encoded the current Unix millisecond timestamp in lowercase base36 for test container names and visible service labels. The Syncthing web/storage test peers share one timestamp code.
- Added regression assertions for encoded names, project labels, and the shared peer suffix. Updated the test index.
- Added the root `TESTING.md` with build-cache behavior, linked-clone image layers, main and focused commands, log inspection, and separate integration-test prerequisites.

## Verification

Initial attempt: `rawrarr-testing-tests-20261018T062901Z-e733a0cb5f18b911` passed all six Node tests and eight of nine Rust tests. The remaining Rust test required age post-quantum identity support. The Debian package was too old, so use upstream age 1.3.2 and verify its release archive checksum during image build.

Successful run: `rawrarr-testing-tests-mvdgotr6` exited 0 in `rawrarr-testing` with networking disabled. All six Node tests passed; all nine Rust unit tests passed, including encrypted config/artwork key generation. Rust dependencies and precompiled test artifacts were reused from the base layers. Database-backed Python tests and host OrbStack peer orchestration were reported as skipped. Logs remain available with `docker --context orbstack logs rawrarr-testing-tests-mvdgotr6`.

Removed the earlier failed-run containers, old `rawrarr-testing-runner:local` image, and prior testing network. At the user's request the project-code test environment was recreated with the passing test container and reusable `rawrarr-testing-base:local` image. A Syncthing peer pair was briefly started after an ambiguous instruction, passed its file and encrypted-config sync check, then was removed when the user clarified that they meant the project-code test container. No peer containers or network remain. The persistent `rawrarr-local` Rawrarr and PostgreSQL containers were verified healthy and left untouched.

The runner reports database-backed Python tests and host-OrbStack peer orchestration as skipped, since they need external fixtures or the host Docker context. They remain separately indexed tests.

## Final state

Complete. The base image and passing test container are retained in the `rawrarr-testing` project. Remaining work is to provision dedicated disposable PostgreSQL fixtures to run the database-backed Python integration tests inside a suitable isolated test environment.
