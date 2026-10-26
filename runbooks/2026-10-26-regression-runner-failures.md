# Fix isolated regression runner failures

Issue [#130](https://github.com/willjasen/rawrarr/issues/130).

## Objective and initial state

Fix the local regression failures and run the available test suites. Work started in `/Users/willjasen/.codex/worktrees/fill-test-coverage-current` on `codex/fill-test-coverage-current`. The first run found the workflow fixture missing from the container snapshot. A follow-up run found two Chromium tests timing out while browser tests ran with the parallel Node suite. During the task, the branch was fast-forwarded to newer `origin/main` at `c73c20b`; that revision already included the `.github/` fixture-copy fix.

Related context: [implemented test coverage](2026-10-26-regression-browser-coverage.md) and [local OrbStack test rerun](2026-10-25-rerun-local-orbstack-tests.md).

## Changes

- Exclude browser-dependent files from the parallel Node run and execute each Playwright file serially in its own browser phase.
- Extend the runner source test to verify browser scheduling. Update the test index with the serial browser phase.
- Update the disposable role test's anonymous-root expectation to match the current public landing page and report container state/exit code when a service fails before port publication, without printing container logs.

## Verification

- Full OrbStack regression command, `python3 scripts/tests/run_rawrarr_tests_in_docker.py --context orbstack --remove-after`: passed on `c73c20b`; Node 110/110, Playwright 28/28 across 14 browser files, Rust 84/84. The earlier two browser timeouts pass when isolated and serial.
- `python3 -m unittest discover -s scripts/tests -p 'test_*.py'`: 35 passed.
- Standalone Python script tests for album sync, media hashing, deploy targets, local Radarr extraction, Tailscale lock safety, node provisioning, Radarr recovery, local Syncthing setup, storage sharing, and UI bundles: 35 passed.
- Read-only coordinator health test passed.
- `test_rawrarr_syncthing_orbstack.py` passed peer connection, folder transfer, encrypted-file equality, and cleanup checks.
- `docker compose -p rawrarr-test-environment -f docker-compose.orbstack-test.yml config --quiet` passed.
- `test_rawrarr_role_deployment.py --apply --image rawrarr:roles-test` remains incompatible with current trust bootstrap: its disposable metadata node exits before serving because the fixture does not initialize the coordinator instance signing root or supply its independently pinned fingerprint. Its current browser flow also needs to provision and use the encrypted root during node approval. The disposable image was rebuilt from this checkout; the test cleanup handler removed its containers and network.
- The standalone configuration integration previously failed at startup because its harness does not set `RAWARR_ROLE` or provide coordinator/node setup. Its error-output change was reverted to preserve secret-safe diagnostics.
- The full multi-node environment was not started: `rawrarr-test-environment` was already active from `/Users/willjasen/.codex/worktrees/fossil-port-forward-tests/rawrarr`. Its containers were left untouched; Compose validation above was read-only.

## Final state and remaining work

The regression runner, standalone Python checks, read-only health check, and isolated Syncthing peer test pass. The manual role and configuration integration harnesses still need to be updated for the current coordinator signing-root and split-role architecture before they can pass end to end. The active full environment was not altered or re-tested. No deployment was performed and no changes were pushed.
