# Rawrarr tests

- `test_rawrarr_mini_export.mjs` — headless Chromium mini-export test using real post-quantum ciphertext from the `age` CLI; verifies browser decryption, HMAC filename matching, local ZIP contents, wrong-key handling, and that the private ID is never sent. Run in the isolated Playwright test image.

The tests below cover Rawrarr application behavior. Some require a disposable PostgreSQL database or Docker with OrbStack; read each test's prerequisites before running it.

- `test_rawrarr_age_setup.mjs` — checks the JPEG XL logo source and PNG fallback, shortened age-identity display, private-key-only copying, identity-derived visuals, and copy/backup acknowledgement gating with character verification temporarily disabled. Requires Node.js; run `node --test scripts/tests/test_rawrarr_age_setup.mjs`.
- `test_rawrarr_age_setup_browser.mjs` — runs the generated-ID and backup acknowledgement flow in headless Chromium, checking that the private key stays off the page, the copy action copies only the key, and Continue stays gated until acknowledgement. Requires the isolated test image built by `run_rawrarr_tests_in_docker.py`; run it inside the test container with `node --test /workspace/scripts/tests/test_rawrarr_age_setup_browser.mjs`.
- `test_rawrarr_docker_healthchecks.mjs` — checks Rawrarr/PostgreSQL readiness checks and unique naming for containers in `rawrarr-testing`. Requires Node.js; run `node --test scripts/tests/test_rawrarr_docker_healthchecks.mjs`.
- `test_rawrarr_brand_case.mjs` — checks that pages and runtime messages display the product name as lowercase. Requires Node.js; run `node --test scripts/tests/test_rawrarr_brand_case.mjs`.
- `test_rawrarr_passkey_ui.mjs` — checks browser-side decoding of WebAuthn API options. Requires Node.js; run `node --test scripts/tests/test_rawrarr_passkey_ui.mjs`.
- `test_rawrarr_configuration.py` — checks encrypted configuration, secret redaction, scanner sidecars, and encrypted poster delivery against an isolated server and disposable PostgreSQL database. Requires Python 3, `psql`, `age`, `age-keygen`, a release binary, and a database ending in `_test`.
- `test_rawrarr_passkey.py` — checks passkey setup, challenge/session handling, and malformed credential rejection against an isolated PostgreSQL database. Requires Python 3, `psql`, a release binary, and a database ending in `_test`.
- `test_rawrarr_syncthing_peers.py` — checks the process-based Syncthing peer flow on systems without Docker. Requires Python 3 and Syncthing.
- `test_rawrarr_syncthing_orbstack.py` — starts isolated web/storage Syncthing containers in the `rawrarr-testing` Docker group and verifies real folder and file transfer. Requires OrbStack's Docker context and Python 3; use `--keep` to leave the labeled test group running for inspection.
- `run_rawrarr_tests_in_docker.py` — creates each uniquely named `rawrarr-testing` runner from `rawrarr-testing-base:local`, copies the current source and tests into its copy-on-write layer, then runs Node and Rust tests offline with retained logs and a final summary naming failed cases. The base contains Chromium, pinned Playwright driver, external tools, and cached Rust dependencies, not project source; it is built only when missing unless `--rebuild-base` is supplied. Container and visible service names include a Unix timestamp encoded in lowercase base36. Run with `python3 scripts/tests/run_rawrarr_tests_in_docker.py`; add `--remove-after` to remove the finished container after displaying its logs. Requires OrbStack's Docker context.

- `test_rawrarr_navigation.mjs` — Chromium regression coverage for Movies/Configuration reload, in-page refresh, history, direct links, fallback, and Settings. Uses the existing isolated test image with Playwright and Chromium; run `node --test /workspace/scripts/tests/test_rawrarr_navigation.mjs` inside it.

- `test_rawrarr_movie_links.mjs` — Chromium checks for reachable, blocked, timed-out, missing-slug and reverse-proxy movie links, including poster/title clicks. Run in the existing isolated test image with Node.js, Playwright, and Chromium.

- `test_rawrarr_quality_profiles.rs` — Rust cache-first quality-profile tests: batch fetch, persistence, warm cache with offline upstream, preserved fields, changed assignments, and failure handling. Run `cargo test --locked --offline --manifest-path rust-arr/Cargo.toml quality_profiles` in the isolated testing image.

- `test_rawrarr_movie_responsiveness.mjs` — Chromium checks that pending Radarr probes do not block rows, refresh, or posters, and failed refreshes retain the previous list. Run in the existing isolated Playwright test image.

- `test_rawrarr_movie_identifiers.mjs` — Chromium checks for clean movie titles, separate TMDb IDs, missing-ID placeholders, column alignment, and omitted movie paths. Run in the isolated Playwright test image.

- `test_rawrarr_metadata_cache.rs` — vault-local ciphertext cache reuse, same-length replacement invalidation, missing files, identity isolation, and concurrent replacement safety. Run `cargo test metadata_cache`.

- `test_setup_local_syncthing.py` — Sanitized local-review bootstrap tests: preserve API key, idempotent folder setup, loopback listener, reject remote peers/unrelated folders. Run `python3 scripts/tests/test_setup_local_syncthing.py`; no Docker required.

- `test_rawrarr_movie_snapshot.rs` — snapshot lifetime, empty libraries, shared client clones, and replacement-client isolation. Run `cargo test movie_snapshot`. The responsiveness browser tests also cover immediate snapshot display and late-response ordering.

- `test_rawrarr_sonarr.rs`: Sonarr series normalization and legacy config compatibility; `cargo test sonarr`.
- `test_rawrarr_arr_settings.rs`: encrypted cross-app save/clear isolation and filename-key preservation; `cargo test arr_settings` (age required).
- `test_rawrarr_shared_library.mjs`: Chromium fixture test proving column changes propagate across Radarr and Sonarr, provider-specific values, and navigation; `node --test scripts/tests/test_rawrarr_shared_library.mjs` in the test image.

- `test_rawrarr_artwork_timing.mjs` — Per-request durations, lazy wait exclusion, missing timing and resource observer collection; run with Node.
- `test_rawrarr_artwork_timing_browser.mjs` — Chromium regression for scrolling to a deferred poster after a wait; run in the isolated Playwright image.
