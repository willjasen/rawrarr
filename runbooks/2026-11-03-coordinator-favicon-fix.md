# Coordinator favicon fix

Issue: https://github.com/willjasen/rawrarr/issues/275

## Change

Replace the externally linked, distorted SVG wordmark with a self-contained square Rawrarr icon. Serve an actual multi-size ICO file from `/favicon.ico` with `image/x-icon`, retain the SVG route, and advertise both formats on the Rawrarr HTML entry pages. Add both icon assets to the published UI bundle so deployed bundles and embedded first-boot fallbacks behave the same.

## Adoption and compatibility

This changes static UI assets and HTTP response metadata only. It does not change persistent data, database schemas, encrypted configuration, or node/coordinator protocols, so no migration is needed. Existing deployments adopt the change when updated with the application image; subscribers do not require a separate state migration.

## Verification

`python3 scripts/tests/test_ui_bundle.py -v` passed all six packaging and updater tests. The isolated regression runner now provides a test-local `playwright` alias for the image's pinned `playwright-core`; the favicon browser test uses the bare import and passed, exercising the alias. The Docker harness fixture test also passes after checking the current `ROOT_FIXTURES` list instead of the obsolete copy layout. `python3 scripts/tests/run_rawrarr_tests_in_docker.py --remove-after` passed all 151 Node.js tests and all 116 Rust unit tests. The full Playwright run had two unrelated failures: a coordinator Sonarr settings timeout and signing-topology desktop visibility. No live deployment was performed; the coordinator's authenticated routes returned HTTP 401 during the initial read-only check.

The isolated regression runner links the image's installed `playwright-core` package as the bare `playwright` import in its test directory. The favicon browser test deliberately uses that bare import so the full runner exercises the alias; the Docker health-check test keeps this package mapping covered.
