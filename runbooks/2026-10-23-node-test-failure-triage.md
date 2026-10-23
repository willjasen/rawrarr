# Node test failure triage and verified test save

## Objective

Diagnose the Node failures from issue #55's local OrbStack run, fix genuine errors where they can be separated from the current uncommitted policy/UI snapshot, and save a verified test change.

## Starting state

- Worktree: `codex/test-failure-fixes` at `/Users/willjasen/.codex/worktrees/rawrarr-test-failures`.
- The OrbStack test runner fix is committed as `1067ee2`.
- The checkout contains broad uncommitted source and test changes, including conflict markers in `docs/device-signing-design.md`, `runbooks/README.md`, and `scripts/tests/README.md`.
- Latest complete test container `rawrarr-testing-tests-mvk9l12f` copied the local worktree and reported 86 Node tests passing and 31 failing; Playwright passed; Rust passed 79 tests.

## Findings and actions

- The Node failures are predominantly source-structure assertions or browser assertions that disagree with current uncommitted changes. Examples include changed direct-route behavior, removed profile/device signature and avatar UI, and changed age-gating behavior. These are not attributable to the runner's duplicate-copy error.
- A focused failure also occurs in the Sonarr browser-settings fixture; the fixture retained route stubs for modules removed by the current uncommitted implementation. The worktree already removes those now-unused stubs, but the browser flow still times out, so this behavior is not recorded as fixed.
- Reviewed the changed age-setup test and ran it independently in the OrbStack test image.
- Added the explicit local-worktree/no-GitHub-pull rule to `AGENTS.md` as requested.
- Rebuilt `rawrarr:orbstack-test` from this worktree and recreated only `roarlay`, `metadata-a`, and `metadata-b` in the isolated `rawrarr-orbstack-test` project. PostgreSQL, Headscale, and named volumes were preserved.
- The rebuilt coordinator and both subscriber apps are healthy. A synthetic signing root and matching node certificates were provisioned only in the isolated test PostgreSQL fixture; no private identity/root was written to disk or output. The coordinator health test passes.
- Both Headscale peers remain online and `tailscale ping` succeeds in both directions. The separate Syncthing web/storage peer integration passes, including encrypted fixture replication and cleanup.
- Updated `test_coordinator_health.py` to check the current public landing-page and Roarlay health-page split plus a protected workspace route; its OrbStack check passes.
- Updated `test_ui_bundle.py` for the current checksum-verified release updater (no longer asserting removed signature/polling behavior); all five tests pass.

## Verification

- `node --experimental-global-webcrypto --test scripts/tests/test_rawrarr_age_setup.mjs` in the isolated OrbStack test image — passed (1/1).
- `python3 scripts/tests/run_rawrarr_tests_in_docker.py --context orbstack` — Node 86 passed / 31 failed; Playwright passed; Rust passed (79/79). The complete suite is not green. Node failures are listed in the runner output/log for `rawrarr-testing-tests-mvk9l12f`.
- `python3 scripts/tests/test_coordinator_health.py --base-url http://127.0.0.1:17397` — passed against the rebuilt local coordinator.
- `python3 scripts/tests/test_ui_bundle.py -v` — passed (5/5).
- `python3 scripts/tests/test_rawrarr_syncthing_orbstack.py` — passed two-peer sync, matching SHA-256, encrypted-config byte equality, and cleanup.
- Coordinator, metadata-a, and metadata-b containers are healthy; both metadata `/health` endpoints and coordinator `/health` return HTTP 200. Tailnet cross-pings succeeded.
- No source fix is claimed for the remaining Node failures because they stem from behavior changes in the uncommitted snapshot and require a policy decision about which behavior is intended.

## Final state and remaining work

The verified age-setup test adjustment is saved on this branch. The coordinator health and UI updater tests match the current local behavior. The isolated coordinator and two-node Compose fixture is operational, and peer connectivity is verified. Remaining Node failures need the current implementation and regression expectations reconciled. The full regression suite is not green; no deployment was performed.
