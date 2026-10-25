# Rerun local OrbStack tests

## Objective and source state

Reread the current `AGENTS.md` and rerun local Rawrarr tests in OrbStack without pulling GitHub state. The tested source was the shared `/Users/willjasen/.binds/sync/Codebox/rawrarr` checkout at `c28e4de` (`main`) plus its existing staged, unstaged, and unmerged local changes. The checkout was already dirty before this test task and contained unresolved conflict markers, including `rust-arr/src/main.rs`. Those changes were left untouched.

## Regression suite

The regression runner defaults to the retired `rawrarr-testing` label. To use the current required project name without editing the shared checkout, invoked it in-process with `PROJECT` overridden to `rawrarr-test-regression`:

```sh
python3 -c 'import scripts.tests.run_rawrarr_tests_in_docker as runner; runner.PROJECT = "rawrarr-test-regression"; raise SystemExit(runner.main())' --context orbstack --rebuild-base
```

The refreshed test image built successfully, including the Rust dependency cache. The final run was `rawrarr-test-regression-tests-mvofg2zu` (2026-10-25 22:59:47–23:01:05 UTC):

- Node: 65 passed, 22 failed.
- Playwright browser: 1 passed.
- Rust: failed to compile because `rust-arr/src/main.rs:2494` contains unresolved Git conflict markers.
- Several Node failures include missing `/workspace/admin/index.html` in the runner's copied fixture, stale source assertions, a test expecting the retired project label, and a subscriber onboarding browser timeout.

The initial run against the stale cached image also failed Rust offline because `crc32fast v1.5.2` was missing. Rebuilding the image addressed that setup issue; the next run exposed the actual source conflict.

## Environment suite

The instructions now require Compose project `rawrarr-test-environment`. That project was absent at the start. Its Compose configuration passed `docker compose -p rawrarr-test-environment -f docker-compose.orbstack-test.yml config --quiet`, but building its Rawrarr services failed at the same unresolved marker in `rust-arr/src/main.rs:2494`. No containers were created for the new environment project. The previous `rawrarr-orbstack-test` containers were stopped and left unchanged; no old project was treated as the current environment.

The standalone Syncthing peer test passed under the regression project label and removed its temporary containers/network/data:

```sh
python3 -c 'import scripts.tests.test_rawrarr_syncthing_orbstack as test; test.PROJECT = "rawrarr-test-regression"; raise SystemExit(test.main())'
```

It verified send-only/receive-only peers, matching SHA-256 for the sample metadata, and byte-identical age-encrypted config with no plaintext in the shared folder.

## Final state and remaining work

The regression test container logs remain available in OrbStack. The transient Syncthing resources were cleaned up. No source files in the shared checkout were edited by this task. The full coordinator-and-two-node environment could not be started until the existing conflict in `main.rs` is resolved by its owner; its remaining test results are therefore unverified.
