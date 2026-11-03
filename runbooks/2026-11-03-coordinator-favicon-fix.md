# Coordinator favicon fix

Issue: https://github.com/willjasen/rawrarr/issues/275

## Change

Replace the externally linked, distorted SVG wordmark with a self-contained square Rawrarr icon. Serve an actual multi-size ICO file from `/favicon.ico` with `image/x-icon`, retain the SVG route, and advertise both formats on the Rawrarr HTML entry pages. Add both icon assets to the published UI bundle so deployed bundles and embedded first-boot fallbacks behave the same.

## Adoption and compatibility

This changes static UI assets and HTTP response metadata only. It does not change persistent data, database schemas, encrypted configuration, or node/coordinator protocols, so no migration is needed. Existing deployments adopt the change when updated with the application image; subscribers do not require a separate state migration.

## Verification

`python3 scripts/tests/test_ui_bundle.py -v` passed all six packaging and updater tests. `python3 scripts/tests/run_rawrarr_tests_in_docker.py --remove-after` passed all 151 Node.js tests and all 116 Rust unit tests; the new favicon browser test and affected subscriber-onboarding browser test also passed. The full Playwright suite reported unrelated failures in the coordinator Sonarr settings timeout, the Docker runner fixture's stale `ROOT / "docs"` assertion, and signing-topology desktop visibility. No live deployment was performed; the coordinator's authenticated routes returned HTTP 401 during the initial read-only check.
