# Testing Rawrarr

This guide explains how to run Rawrarr's current tests, with the self-contained Node.js and Rust suites running inside the reusable Docker test base. The runner creates a fresh container for every invocation, prints the test output to its logs, and leaves the stopped container available for inspection.

## Test environment

Testing containers belong to the Docker project group `rawrarr-testing`. Keep them separate from `rawrarr-local`, which is the persistent local application environment. The runner uses the Docker context `orbstack` by default.

The local image `rawrarr-testing-base:local` is the reusable test base. It contains:

- Rust 1.85 and the Rawrarr Rust source and migrations.
- Node.js for the browser-side and source-structure tests.
- The locked Cargo crate cache and precompiled Rust test executable.
- The upstream `age` 1.3.2 binaries, including `age-keygen`, verified against the release archive's SHA-256 checksum.
- The test files and the test entrypoint.

The Dockerfile downloads the Rust crates once in a layer based on `Cargo.toml` and `Cargo.lock`. Application files and tests are added in later layers. Docker reuses the unchanged layers on later builds; after the first complete build, a test run can usually start from the local base without downloading or compiling dependencies again. If the crate manifest or lockfile changes, Docker refreshes the dependency layer. If Rust source changes, Docker rebuilds the Rust test artifact layer. Changes to the test scripts do not invalidate those earlier Rust layers.

This works like a linked clone: Docker stores and shares the base's immutable image layers, while each test container gets a separate writable layer. Every run uses the same base tag but has its own generated name, timestamp-derived identifier, and `rawrarr-testing` service label. The identifier is the current Unix timestamp in milliseconds encoded in lowercase base36. The container has no network and does not mount the live repository, so it runs from the source snapshot baked into its image.

## Run the main test suite

From the repository root, run:

```sh
python3 scripts/tests/run_rawrarr_tests_in_docker.py
```

The script checks the selected Docker context, prepares a small temporary build context, builds or reuses `rawrarr-testing-base:local`, and creates and starts one uniquely named container. It then follows the container logs until the tests finish and returns the test container's exit status. A zero exit status means the included Node.js and Rust suites passed.

To use a different Docker context explicitly:

```sh
python3 scripts/tests/run_rawrarr_tests_in_docker.py --context orbstack
```

To display all output and then remove that run's stopped container:

```sh
python3 scripts/tests/run_rawrarr_tests_in_docker.py --remove-after
```

By default, completed containers are retained so their output can be reviewed later. The base image remains tagged `rawrarr-testing-base:local`; no test runner container needs to download Rust crates when launched. The first base build does need network access to fetch OS packages, the age release archive, and locked Cargo crates. Test execution itself has networking disabled.

## Inspect a run

The runner prints the exact generated container name, for example:

```text
TEST CONTAINER: rawrarr-testing-tests-mvdgmyn4
```

Use that exact name to read the log again or inspect the result:

```sh
docker --context orbstack logs rawrarr-testing-tests-mvdgmyn4
docker --context orbstack inspect --format '{{.State.Status}} {{.State.ExitCode}}' rawrarr-testing-tests-mvdgmyn4
docker --context orbstack inspect --format '{{json .Config.Labels}}' rawrarr-testing-tests-mvdgmyn4
```

To list every container currently associated with the test group, including stopped runs:

```sh
docker --context orbstack ps -a \
  --filter label=com.docker.compose.project=rawrarr-testing \
  --format 'table {{.Names}}\t{{.Status}}\t{{.Image}}'
```

The human-readable service label also contains the base36 suffix. The persistent `rawrarr-local` containers use a different project label and are not targets of this runner.

## What the main container runs

The entrypoint executes both suites and prints a summary at the end:

1. All `scripts/tests/*.mjs` Node.js tests, including setup UI behavior, lowercase product branding, passkey decoding, readiness health checks, and test-container naming.
2. The locked Rust unit tests using `cargo test --locked --offline`.

The runner reports database-backed Python integration tests and host-OrbStack peer synchronization as skipped. They need disposable external fixtures or a host Docker context, so they are separate procedures below. A skip message is not a passing result for those integrations.

## Run focused tests inside the test base

Once the test base has been built by the main runner, focused Node and Rust tests can also run in fresh, offline containers from the same image:

```sh
docker --context orbstack run --rm --network none \
  --entrypoint node rawrarr-testing-base:local \
  --test /workspace/scripts/tests/test_rawrarr_age_setup.mjs
```

```sh
docker --context orbstack run --rm --network none \
  --entrypoint cargo rawrarr-testing-base:local \
  test --locked --offline --manifest-path /workspace/rust-arr/Cargo.toml
```

These one-off focused containers are removed after completion and their output appears directly in the terminal. For retained output under the `rawrarr-testing` project, use the main runner.

## Run the Syncthing peer-container integration

The two-container OrbStack integration checks real file and encrypted configuration synchronization. It is deliberately separate from the offline unit-test container because it needs Docker networking and host-side API checks. Run it from the repository root:

```sh
python3 scripts/tests/test_rawrarr_syncthing_orbstack.py
```

It creates a uniquely named web peer, storage peer, and network. Their names share the same timestamp-derived base36 suffix, and their displayed service labels include that suffix. The script uses temporary fixture directories and removes its containers, network, and temporary data after the check. Add `--keep` to preserve the labeled test group and data for inspection:

```sh
python3 scripts/tests/test_rawrarr_syncthing_orbstack.py --keep
```

The host needs the `orbstack` Docker context, Python 3, and age 1.3.0 or later with post-quantum (`-pq`) key generation support. The test pulls the pinned Syncthing image and needs temporary free localhost ports.

## Database-backed Python integration tests

`scripts/tests/test_rawrarr_configuration.py` and `scripts/tests/test_rawrarr_passkey.py` require an isolated disposable PostgreSQL database whose name ends in `_test`, a release build of Rawrarr, Python 3, and the tools each test lists in `scripts/tests/README.md`. Use only a dedicated test database and test credentials. Do not point these tests at the persistent `rawrarr-local` database or real user data.

The process-based Syncthing peer check, `scripts/tests/test_rawrarr_syncthing_peers.py`, requires Python 3 and Syncthing installed on the host. It does not replace the OrbStack container-to-container sync integration.

## Test inventory

See [`scripts/tests/README.md`](scripts/tests/README.md) for the current test-by-test inventory, dependencies, and coverage. See [`scripts/tests/Dockerfile`](scripts/tests/Dockerfile), [`scripts/tests/run-tests-in-container.sh`](scripts/tests/run-tests-in-container.sh), and [`scripts/tests/run_rawrarr_tests_in_docker.py`](scripts/tests/run_rawrarr_tests_in_docker.py) for the image and runner implementation.
