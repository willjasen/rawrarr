# Rawrarr tests

The tests below cover Rawrarr application behavior. Some require a disposable PostgreSQL database or Docker with OrbStack; read each test's prerequisites before running it.

- `test_rawrarr_age_setup.mjs` — checks shortened age-identity display, copy-only private identity, identity-derived visuals, and backup confirmation gating. Requires Node.js; run `node --test scripts/tests/test_rawrarr_age_setup.mjs`.
- `test_rawrarr_passkey_ui.mjs` — checks browser-side decoding of WebAuthn API options. Requires Node.js; run `node --test scripts/tests/test_rawrarr_passkey_ui.mjs`.
- `test_rawrarr_configuration.py` — checks encrypted configuration, secret redaction, scanner sidecars, and encrypted poster delivery against an isolated server and disposable PostgreSQL database. Requires Python 3, `psql`, `age`, `age-keygen`, a release binary, and a database ending in `_test`.
- `test_rawrarr_passkey.py` — checks passkey setup, challenge/session handling, and malformed credential rejection against an isolated PostgreSQL database. Requires Python 3, `psql`, a release binary, and a database ending in `_test`.
- `test_rawrarr_syncthing_peers.py` — checks the process-based Syncthing peer flow on systems without Docker. Requires Python 3 and Syncthing.
- `test_rawrarr_syncthing_orbstack.py` — starts isolated web/storage Syncthing containers and verifies real folder and file transfer. Requires OrbStack's Docker context and Python 3.
