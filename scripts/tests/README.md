- `test_rawrarr_artwork_cache.rs` — checks disabled-cache file reads and disk fallback when Redis is unreachable; runs with the Rust suite or `cargo test artwork_cache` in the isolated test image.

- `test_rawrarr_coordinator_arr_links.mjs` — checks the coordinator's encrypted Radarr slug flow, direct metadata route, browser-only reachability gate, and year-cell icon wiring for Movies and TV. Run with `node --test scripts/tests/test_rawrarr_coordinator_arr_links.mjs`.
# Rawrarr tests

- `test_rawrarr_arr_summary.mjs` — checks that Radarr and Sonarr use the shared configurable four-card summary renderer. Run with `node --test scripts/tests/test_rawrarr_arr_summary.mjs`.

- `test_rawrarr_metadata_sizes.mjs` — checks the metadata API and Movies/TV cards report separate media-subtree sizes. Run with `node --test scripts/tests/test_rawrarr_metadata_sizes.mjs`.

- `test_rawrarr_sonarr_completion_ui.mjs` — confirms the TV view hides scrape completion and successful-load messages after the Sonarr artwork cache is complete, while retaining errors. Run with `node --test scripts/tests/test_rawrarr_sonarr_completion_ui.mjs`.

- `test_rawrarr_direct_route.mjs` — static regression checks for Tailscale direct metadata routing, the short-lived node-scoped session, visible automatic selection, and coordinator fallback. Run with `node --test scripts/tests/test_rawrarr_direct_route.mjs`.
- `test_rawrarr_device_routes.mjs` — checks the shared all-device route-status helper for Direct, Relay, and Offline states, Redis cache reachability in measured node latency, cached probes, and explicit refresh. Run with `node --test scripts/tests/test_rawrarr_device_routes.mjs`.
- `test_rawrarr_device_routes.mjs` — checks the shared all-device route-status helper for Direct, Relay, and Offline states, Redis cache reachability in measured node latency, probe caching, and explicit refresh. Run with `node --test scripts/tests/test_rawrarr_device_routes.mjs`.

- `test_rawrarr_node_network_isolation.mjs` — checks that metadata/storage have no published host ports and share a Tailscale namespace with loopback-only listeners and relay targets. Requires Node.js; run `node --test scripts/tests/test_rawrarr_node_network_isolation.mjs`.

- `test_rawrarr_mini_export.mjs` — headless Chromium mini-export test using real post-quantum ciphertext from the `age` CLI; verifies browser decryption, HMAC filename matching, local ZIP contents, wrong-key handling, and that the private ID is never sent. Run in the isolated Playwright test image.

- `test_rawrarr_role_deployment.py` — disposable Docker coordinator/metadata/storage integration, real Chromium virtual-passkey flow for account-linked coordinator-origin credential addition (via `helpers/role_passkey_browser.mjs`), session scoping, logout, role mismatch and outage. Requires `rawrarr:roles-test`, PostgreSQL 17, Caddy, and the existing testing base image; dry-run by default, run with `--apply`.
- `test_rawrarr_storage_node.rs` — validates daily storage scan interval settings, bounded movie sample filtering, opaque cache keys, and standard MD5/SHA-1/SHA-256 output. Runs through the Rust test suite.
- `test_rawrarr_storage_scan_ui.mjs` — checks the Devices scan actions, bounded 1–2 GB movie sample, progress routes, and storage-only coordinator relay gate. Run `node --test scripts/tests/test_rawrarr_storage_scan_ui.mjs`.
- `test_rawrarr_coordinator_workspace.mjs` — coordinator passkey landing page, owner-scoped node list and relay, roarlay/apex UI labels and Compose names, browser-profile age-key sharing and clearing, persisted age-ID rehydration and Movies unlock behavior, browser-side cluster config decryption and secret redaction, authenticated coordinator JPEG XL avatar delivery with PNG fallback from a public-age-ID digest, Tailscale SOCKS routing, delegated node sessions, accessible online/offline indicators, configured Radarr/Sonarr media-tab form visibility, and Sonarr series rows/summary rendering from the direct metadata API. Run `node --test scripts/tests/test_rawrarr_coordinator_workspace.mjs`; Rust avatar tests cover deterministic dinosaur traits, rarity, six scenes, spike/wings rules, and raster output formats.

- `test_provision_node.py` — dry-run, protected token/verifier, duplicate preservation, and invalid scope tests; run `python3 scripts/tests/test_provision_node.py`.
- `test_rawrarr_node_roles.rs` and `test_rawrarr_node_auth.rs` — role/media boundaries and authentication proxy/transport allowlists; run `cargo test node_` in the isolated test image.

The tests below cover Rawrarr application behavior. Some require a disposable PostgreSQL database or Docker with OrbStack; read each test's prerequisites before running it.

- `test_rawrarr_artwork_source.mjs` — checks the Radarr movie API's shared-cache source classification and the Movies table artwork-source column. Requires Node.js; run `node --test scripts/tests/test_rawrarr_artwork_source.mjs`.

- `test_rawrarr_age_setup.mjs` — checks the JPEG XL logo source and PNG fallback, shortened age-identity display, private-key-only copying, identity-derived visuals, and copy/backup acknowledgement gating with character verification temporarily disabled. Requires Node.js; run `node --test scripts/tests/test_rawrarr_age_setup.mjs`.
- `test_rawrarr_age_setup_browser.mjs` — runs the generated-ID and backup acknowledgement flow in headless Chromium, checking that the private key stays off the page, the copy action copies only the key, and Continue stays gated until acknowledgement. Requires the isolated test image built by `run_rawrarr_tests_in_docker.py`; run it inside the test container with `node --test /workspace/scripts/tests/test_rawrarr_age_setup_browser.mjs`.
- `test_rawrarr_docker_healthchecks.mjs` — checks Rawrarr/PostgreSQL readiness checks and unique naming for containers in `rawrarr-testing`. Requires Node.js; run `node --test scripts/tests/test_rawrarr_docker_healthchecks.mjs`.
- `test_rawrarr_brand_case.mjs` — checks that pages and runtime messages display the product name as lowercase. Requires Node.js; run `node --test scripts/tests/test_rawrarr_brand_case.mjs`.
- `test_rawrarr_passkey_ui.mjs` — checks browser-side decoding of WebAuthn API options and that first setup generates a dinosaur-themed account name without asking for one. Requires Node.js; run `node --test scripts/tests/test_rawrarr_passkey_ui.mjs`.
- `test_rawrarr_profile_color.mjs` — checks profile color and wing-word labels match the deterministic dinosaur avatar digest. Requires Node.js; run `node --test scripts/tests/test_rawrarr_profile_color.mjs`.
- `test_rawrarr_coordinator_passkey.mjs` — checks the signed-in metadata-to-coordinator one-time passkey handoff, origin-local ceremony page, account association, and RP configuration. Requires Node.js; run `node --test scripts/tests/test_rawrarr_coordinator_passkey.mjs`.
- `test_rawrarr_configuration.py` — checks encrypted configuration, secret redaction, scanner sidecars, and encrypted poster delivery against an isolated server and disposable PostgreSQL database. Requires Python 3, `psql`, `age`, `age-keygen`, a release binary, and a database ending in `_test`.
- `test_rawrarr_passkey.py` — checks passkey setup, challenge/session handling, and malformed credential rejection against an isolated PostgreSQL database. Requires Python 3, `psql`, a release binary, and a database ending in `_test`.
- `test_local_radarr.py` — checks safe extraction of the node-local Radarr config archive and summary output without Docker, Radarr, or secret/path output. Run `python3 scripts/tests/test_local_radarr.py`.
- `test_rawrarr_syncthing_peers.py` — checks the process-based Syncthing peer flow on systems without Docker. Requires Python 3 and Syncthing.
- `test_rawrarr_syncthing_orbstack.py` — starts isolated web/storage Syncthing containers in the `rawrarr-testing` Docker group and verifies real folder and file transfer. Requires OrbStack's Docker context and Python 3; use `--keep` to leave the labeled test group running for inspection.
- `run_rawrarr_tests_in_docker.py` — creates each uniquely named `rawrarr-testing` runner from `rawrarr-testing-base:local`, copies the current source and tests into its copy-on-write layer, then runs Node and Rust tests offline with retained logs and a final summary naming failed cases. The base contains Chromium, pinned Playwright driver, external tools, and cached Rust dependencies, not project source; it is built only when missing unless `--rebuild-base` is supplied. Container and visible service names include a Unix timestamp encoded in lowercase base36. Run with `python3 scripts/tests/run_rawrarr_tests_in_docker.py`; add `--remove-after` to remove the finished container after displaying its logs. Requires OrbStack's Docker context.
- `run-tests-in-container.sh` — runs the Node.js, Playwright, and Rust unit suites in an isolated test runner. Invoked by `run_rawrarr_tests_in_docker.py`.

- `run_rawrarr_tests_in_docker.py` — the final container summary includes test start time, stop time, and elapsed seconds. Each invocation checks whether cleanup is due; at most once per hour it removes only stopped `tests-*` containers in the `rawrarr-testing` project that were created more than one hour ago. Preview candidates without deleting them with `python3 scripts/tests/run_rawrarr_tests_in_docker.py --cleanup-dry-run`.

- `test_rawrarr_navigation.mjs` — Chromium regression coverage for Movies/Configuration reload, in-page refresh, history, direct links, fallback, and Settings. Uses the existing isolated test image with Playwright and Chromium; run `node --test /workspace/scripts/tests/test_rawrarr_navigation.mjs` inside it.

- `test_rawrarr_movie_links.mjs` — Chromium checks for reachable, blocked, timed-out, missing-slug and reverse-proxy movie links, including poster/title clicks. Run in the existing isolated test image with Node.js, Playwright, and Chromium.

- `test_rawrarr_quality_profiles.rs` — Rust cache-first quality-profile tests: batch fetch, persistence, warm cache with offline upstream, preserved fields, changed assignments, and failure handling. Run `cargo test --locked --offline --manifest-path rust-arr/Cargo.toml quality_profiles` in the isolated testing image.

- `test_rawrarr_filename_secret.rs` — Rust tests for random key generation, validation, encrypted persistence, legacy migration, peer consistency, Radarr update/removal preservation, and plaintext-mirror exclusion. Included in the isolated Rust suite.
- `test_rawrarr_config_schema.rs` — Rust tests for legacy flat config migration to schema v1, service-array persistence, and unsupported-version rejection. Included in the isolated Rust suite.
- `test_rawrarr_hmac_metadata_filenames.rs` — checks domain-separated opaque movie filenames, byte-identical ciphertext migration, and decryption after migration. Included in the isolated Rust suite.

- `test_rawrarr_movie_responsiveness.mjs` — Chromium checks that pending Radarr probes do not block rows, refresh, or posters, and failed refreshes retain the previous list. Run in the existing isolated Playwright test image.

- `test_rawrarr_movie_identifiers.mjs` — Chromium checks for clean movie titles, separate TMDb IDs, missing-ID placeholders, column alignment, and omitted movie paths. Run in the isolated Playwright test image.

- `test_rawrarr_metadata_cache.rs` — vault-local ciphertext cache reuse, same-length replacement invalidation, missing files, identity isolation, and concurrent replacement safety. Run `cargo test metadata_cache`.

- `test_setup_local_syncthing.py` — Sanitized local-review bootstrap tests: preserve API key, idempotent folder setup, enable global discovery/public relays/NAT, reject remote peers and unrelated folders. Run `python3 scripts/tests/test_setup_local_syncthing.py`; no Docker required.

- `test_rawrarr_movie_snapshot.rs` — snapshot lifetime, empty libraries, shared client clones, and replacement-client isolation. Run `cargo test movie_snapshot`. The responsiveness browser tests also cover immediate snapshot display and late-response ordering.

- `test_rawrarr_sonarr.rs`: Sonarr series normalization and legacy config compatibility; `cargo test sonarr`.
- `test_rawrarr_sonarr_metadata.rs`: TVDB-domain-separated HMAC names and age-encrypted Sonarr metadata/artwork storage; run `cargo test sonarr_metadata` in the isolated test image.
- `secure_config::tests::encrypted_config_and_artwork_are_shareable_age_ciphertexts` — encrypted 512 KiB artwork round-trip regression for large age subprocess pipe deadlocks; run `cargo test --locked --manifest-path rust-arr/Cargo.toml secure_config::tests::encrypted_config_and_artwork_are_shareable_age_ciphertexts -- --exact` in the isolated Rust test image.
- `test_rawrarr_sonarr_scrape.mjs`: metadata-node Sonarr backfill startup, sequential cache-first scraping, immediate save notification, six-hour repeat, allowlisted TVDB artwork fallback, and TV metadata storage boundaries; run `node --test scripts/tests/test_rawrarr_sonarr_scrape.mjs`.
- `test_rawrarr_arr_settings.rs`: encrypted cross-app save/clear isolation and filename-key preservation; `cargo test arr_settings` (age required).
- `test_rawrarr_sonarr_settings.mjs`: Chromium flow for entering a Sonarr URL/API key, saving, clearing the password field after save, reloading only the URL, and clearing the saved connection; run in the isolated Playwright test image.
- `test_rawrarr_coordinator_sonarr_settings.mjs`: Chromium flow for Radarr and Sonarr forms on Movies, TV, and Settings; saved URL/key-state confirmation without exposing API keys; age unlock/browser retention; direct metadata save/read-back/clear; and verifying credentials bypass roarlay. Run in the isolated Playwright test image.
- `test_rawrarr_shared_library.mjs`: Chromium fixture test proving column changes propagate across Radarr and Sonarr, provider-specific values, and navigation; `node --test scripts/tests/test_rawrarr_shared_library.mjs` in the test image.
- `test_rawrarr_node_enrollment.rs`: strict pending-node payload, age-identity exclusion, and origin/ID validation; included in the isolated Rust suite.
- `test_rawrarr_node_approval_ui.mjs`: signed-in, account-scoped pending-node approval UI plus enrollment-only metadata pass-through and bounded bearer-token forwarding; included in the isolated Node.js suite.
- `test_rawrarr_device_names.mjs` — Verifies device names stay in age config, browser encryption/decryption keeps names out of coordinator service storage, and the coordinator uses a dedicated Devices tab with inline name editing; run with `node --test scripts/tests/test_rawrarr_device_names.mjs`.

- `test_rawrarr_artwork_timing.mjs` — Per-request durations, lazy wait exclusion, missing timing and resource observer collection; run with Node.
- `test_rawrarr_artwork_timing_browser.mjs` — Chromium regression for scrolling to a deferred poster after a wait; run in the isolated Playwright image.
- `test_rawrarr_artwork_backfill.mjs` — Metadata-node background scraping, sequential requests, encrypted cache reuse, and Movies progress bar; run `node --test scripts/tests/test_rawrarr_artwork_backfill.mjs`.

- `test_coordinator_health.py` — Read-only live coordinator health-page and protected-auth checks; run `python3 scripts/tests/test_coordinator_health.py --base-url https://rawrarr.risk-mermaid.ts.net`. Requires Python 3.

- `test_rawrarr_device_signing_crypto.mjs` — checks owner-side ML-DSA-65 root creation, age-encrypted backup, local approval signing, root fingerprint matching, and tamper rejection; run with `node --test scripts/tests/test_rawrarr_device_signing_crypto.mjs`.
`test_rawrarr_library_detail_link_layout.mjs` checks that Movies and TV display year and Arr detail links in the artwork/title column. Run with `node --test scripts/tests/test_rawrarr_library_detail_link_layout.mjs`.

`test_rawrarr_library_detail_link_layout.mjs` checks that Movies and TV display year and Arr detail links in the artwork/title column. Run with `node --test scripts/tests/test_rawrarr_library_detail_link_layout.mjs`.
