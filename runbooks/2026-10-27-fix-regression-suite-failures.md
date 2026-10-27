# Fix linked regression suite failures

Issue: [#223](https://github.com/willjasen/rawrarr/issues/223)

## Findings and changes

- The test clone omitted `stripe-service/`, so its static Stripe billing tests failed to open the service source and migration fixtures. The runner now copies that directory into each clone.
- Several Playwright fixtures did not mark Radarr or Sonarr as configured. Current UI behavior correctly hides the library until the service settings endpoint reports `configured: true`; tests now provide that state when asserting library behavior.
- The movie responsiveness regression stalled the settings API instead of the external Radarr reachability probe. It now gates the cross-origin probe and verifies movies and posters still render while that probe is pending.
- The Sonarr test checked a status message inside the setup panel after the panel was hidden on successful configuration. It now waits for the status text independent of visibility and reopens the panel through the configured TV library before clearing the connection.
- Configured Movies and TV libraries now expose a Connection settings button so the existing encrypted update and clear actions remain reachable after setup.
- Navigation coverage now expects the Settings button's actual Configuration view and verifies Radarr settings can be reopened.
- Updated visible auth copy to keep the product name lowercase.

## Verification

- `node --test scripts/tests/test_rawrarr_docker_healthchecks.mjs scripts/tests/test_rawrarr_brand_case.mjs` — passed, 5/5.
- `sh -n scripts/tests/prepare-test-clone.sh scripts/tests/wait-for-test-trigger.sh`, `python3 -m py_compile scripts/tests/run_rawrarr_tests_in_docker.py`, and `git diff --check` — passed.
- `python3 scripts/tests/run_rawrarr_tests_in_docker.py --remove-after` — passed on OrbStack `linux/aarch64`: Node.js passed (121/121), all serial Playwright browser files passed (including movie responsiveness/navigation, Sonarr clearing, Stripe service fixture tests, and ETH/ENS regressions), and Rust passed (106/106). Python integration checks were skipped as documented by the runner; they require disposable PostgreSQL, age, or Syncthing fixtures.
