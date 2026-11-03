# Test fossil host port isolation

## Objective and tracking

Add regression and OrbStack environment checks proving subscriber containers have no Docker-published host ports. GitHub issue [#126](https://github.com/willjasen/rawrarr/issues/126) tracks the coverage.

## Initial state

- Started from fetched `origin/main` at `e9bd1348` in a dedicated worktree on `codex/fossil-port-forward-tests`.
- The generated subscriber Compose regression did not explicitly reject `ports:` mappings. The static node-network test checked only a few known port strings.
- The OrbStack Arr integration checked one fossil's app behavior but did not inspect Docker port bindings for either subscriber or its Tailscale sidecar.
- The `rawrarr-test-environment` Compose project was not running at inspection time. Existing test-state files remain outside the repository.
- The first environment startup hit an occupied fixed loopback port. A fresh coordinator database also had no signing root; the test now creates a disposable public-only root fixture when needed.

## Changes

- Make generated fossil and boneyard Compose regression coverage reject any `ports:` section.
- Check the actual subscriber service blocks in the base, metadata, storage, and two-node OrbStack test Compose definitions for port publishing.
- Use the `rawrarr-test-environment` project for the OrbStack integration. Its `--start` path now starts both metadata nodes and checks live Docker `HostConfig.PortBindings` for both fossils and their Tailscale sidecars.
- Select an available loopback coordinator port on startup, preserving an existing project mapping on reruns. Pass the selected port through the isolated Compose environment.
- Create a synthetic Headscale user and reusable one-hour auth keys for the two isolated clients. Store them in owner-only files under the existing test-state directory and pass them to Docker as secrets.
- Add `--ports-only` for a repeatable read-only inspection of an already-created environment, even if its subscriber services are stopped or unhealthy.
- If the isolated database has no signing root, generate an ML-DSA test keypair in memory, save only its public key and fingerprint in the test database, and erase the private key from process memory. This root is test-only and is not an owner recovery backup.
- Keep the coordinator's explicitly loopback-bound test port outside the subscriber assertion.

The standalone port-only command does not need the test key files or Node dependencies. To start the wider environment from a clean checkout, install the pinned web-export dependencies with `npm ci --prefix rust-arr/web-export` first.

## Verification

- Passed: `node --test scripts/tests/test_rawrarr_subscriber_compose.mjs scripts/tests/test_rawrarr_node_network_isolation.mjs` (8 tests).
- Passed: `python3 -m py_compile scripts/tests/test_orbstack_arr_connectivity.py`.
- Passed: `docker compose -p rawrarr-test-environment -f docker-compose.orbstack-test.yml config --quiet`.
- Passed against the Docker-created environment containers: `python3 scripts/tests/test_orbstack_arr_connectivity.py --ports-only` confirmed no host port bindings on metadata-a, metadata-b, tailscale-a, or tailscale-b.
- The full `--start` integration did not complete. It created and retained the isolated project, initialized its test-only public root, generated protected one-hour Headscale keys, and brought up both Tailscale sidecars plus the coordinator/Arr services. Both subscriber containers then exited because the legacy approved-node fixture has no signing keys for the current signed-request protocol; Arr API and peer-ping checks remain unverified. The port-binding assertion passed before the subscriber health checks.
- `git diff --check` passed.

## Final state and remaining work

The focused regression and runtime port-isolation checks pass. The `rawrarr-test-environment` project remains available with its coordinator, database, Headscale, Arr services, and both Tailscale clients running; the two subscriber application containers are stopped because the fixture's preapproved nodes are not upgraded to signed node credentials. Add a test-only signed-approval fixture before expecting the full Arr integration to pass. The focused environment port-isolation check is independently runnable with `python3 scripts/tests/test_orbstack_arr_connectivity.py --ports-only`. No production or subscriber host configuration was changed.
