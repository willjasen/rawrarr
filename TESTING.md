# Testing Rawrarr

This guide explains how to run Rawrarr's current tests, with the self-contained Node.js, Playwright browser, and Rust suites running inside the reusable Docker test base. The runner creates a fresh container for every invocation, prints the test output to its logs, and leaves the stopped container available for inspection.

## Test environment

Testing containers belong to the Docker project group `rawrarr-testing`. Keep them separate from `rawrarr-local`, which is the persistent local application environment. The runner uses the Docker context `orbstack` by default.

The local image `rawrarr-testing-base:local` is the reusable dependency base. Each run clones it into a newly named container using Docker's copy-on-write image layers, copies the current project source into that container, and then starts the tests. The base contains:

- Rust 1.85.
- Node.js for the browser-side and source-structure tests.
- Chromium and `playwright-core` 1.55.0 for the separate browser suite.
- The locked Cargo crate cache and compiled dependency artifacts used by Rust tests.
- The upstream `age` 1.3.2 binaries, including `age-keygen`, verified against the release archive's SHA-256 checksum.

The base downloads the Rust crates and compiles dependency artifacts from a placeholder application. It does not contain Rawrarr source or project tests. The runner builds the base only when it is missing. It does not refresh an existing base when project source or tests change. Use `--rebuild-base` deliberately after changing the Rust toolchain, Cargo lockfile, or base-image dependencies. The current Rust source, migrations, Compose file, and test files (including the runner script) are copied into each new container before it starts, so each run tests the current checkout.

Each runner works like a linked clone: Docker stores and shares the base's immutable image layers, while each test container gets a separate writable layer. Every run has its own generated name, timestamp-derived identifier, and `rawrarr-testing` service label. The identifier is the current Unix timestamp in milliseconds encoded in lowercase base36. The container has no network and does not mount the live repository; it tests the source snapshot copied into its own writable layer.

## Run the main test suite

From the repository root, run:

```sh
python3 scripts/tests/run_rawrarr_tests_in_docker.py
```

The script checks the selected Docker context, creates or reuses `rawrarr-testing-base:local`, creates a uniquely named container from that image, copies in the current project snapshot, and starts the runner. It then follows the container logs until the tests finish and returns the test container's exit status. A zero exit status means the included Node.js, Playwright browser, and Rust suites passed.

To use a different Docker context explicitly:

```sh
python3 scripts/tests/run_rawrarr_tests_in_docker.py --context orbstack
```

To display all output and then remove that run's stopped container:

```sh
python3 scripts/tests/run_rawrarr_tests_in_docker.py --remove-after
```

To refresh the reusable base after changing its dependencies or toolchain:

```sh
python3 scripts/tests/run_rawrarr_tests_in_docker.py --rebuild-base
```

By default, completed containers are retained so their output can be reviewed later. The base image remains tagged `rawrarr-testing-base:local`; source changes never rebuild it, and runner containers do not download dependencies. The first base build does need network access to fetch OS packages, the age release archive, and locked Cargo crates. Test execution itself has networking disabled.

The container summary reports when test execution started and stopped, plus elapsed seconds. On each normal invocation, the runner checks a local timestamp under `~/Library/Application Support/Rawrarr/`; if cleanup is due, it removes only stopped `tests-*` containers in `rawrarr-testing` that are older than one hour. Preview candidates at any time with `python3 scripts/tests/run_rawrarr_tests_in_docker.py --cleanup-dry-run`; this does not update the cleanup timestamp or remove containers.

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

The entrypoint executes three suites and prints a summary at the end:

1. Node.js `scripts/tests/*.mjs` tests except the separate browser test, covering setup UI behavior, product branding, passkey decoding, health-check conventions, and test-container naming.
2. `test_rawrarr_age_setup_browser.mjs` using Playwright and Chromium with a local fixture server and mocked API responses. This checks browser behavior, not real passkey authentication or a live backend.
3. The locked Rust unit tests using `cargo test --locked --offline`. The runner first cleans the application package to avoid reusing the base image's placeholder application artifact.

The final summary reports each suite's result and extracts failing test names or compiler errors while retaining detailed logs above it. Any failed suite produces a nonzero exit status.

The runner reports database-backed Python integration tests and host-OrbStack peer synchronization as skipped. They need disposable external fixtures or a host Docker context, so they are separate procedures below. A skip message is not a passing result for those integrations.

## Focused checks and source snapshots

The dependency base contains a placeholder Rust application and no project test files. Do not run a project test directly from `rawrarr-testing-base:local`: a Node test path will be missing, and a Cargo invocation may test only the placeholder. Use the main runner above for container verification of the current checkout; it copies source into each fresh container before executing all three suites. The runner currently has no focused-test selector.

For a quick focused check on a host with Node.js installed, run from the repository root:

```sh
node --test scripts/tests/test_rawrarr_age_setup.mjs
```

This command reads the current local source. The Playwright test has container-specific Chromium and module paths, so run it through the main container runner. If an older base lacks Chromium or Playwright, refresh it with `--rebuild-base`.

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
