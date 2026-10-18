# Rawrarr tests

The tests below cover Rawrarr application behavior. Some require a disposable PostgreSQL database or Docker with OrbStack; read each test's prerequisites before running it.

- `test_rawrarr_age_setup.mjs` — checks the JPEG XL logo source and PNG fallback, shortened age-identity display, private-key-only copying, random character challenge, identity-derived visuals, and backup confirmation gating. Requires Node.js; run `node --test scripts/tests/test_rawrarr_age_setup.mjs`.
- `test_rawrarr_docker_healthchecks.mjs` — checks that the Rawrarr image and PostgreSQL Compose service define readiness checks. Requires Node.js; run `node --test scripts/tests/test_rawrarr_docker_healthchecks.mjs`.
- `test_rawrarr_brand_case.mjs` — checks that pages and runtime messages display the product name as lowercase. Requires Node.js; run `node --test scripts/tests/test_rawrarr_brand_case.mjs`.
- `test_rawrarr_passkey_ui.mjs` — checks browser-side decoding of WebAuthn API options. Requires Node.js; run `node --test scripts/tests/test_rawrarr_passkey_ui.mjs`.
- `test_rawrarr_configuration.py` — checks encrypted configuration, secret redaction, scanner sidecars, and encrypted poster delivery against an isolated server and disposable PostgreSQL database. Requires Python 3, `psql`, `age`, `age-keygen`, a release binary, and a database ending in `_test`.
- `test_rawrarr_passkey.py` — checks passkey setup, challenge/session handling, and malformed credential rejection against an isolated PostgreSQL database. Requires Python 3, `psql`, a release binary, and a database ending in `_test`.
- `test_rawrarr_syncthing_peers.py` — checks the process-based Syncthing peer flow on systems without Docker. Requires Python 3 and Syncthing.
- `test_rawrarr_syncthing_orbstack.py` — starts isolated web/storage Syncthing containers in the `rawrarr-testing` Docker group and verifies real folder and file transfer. Requires OrbStack's Docker context and Python 3; use `--keep` to leave the labeled test group running for inspection.
