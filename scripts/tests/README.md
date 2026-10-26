- `test_rawrarr_library_text_cache.mjs` — checks node-scoped browser library text caching, immediate Movies/TV rendering, live refresh, and separately loaded artwork. Run with `node --test scripts/tests/test_rawrarr_library_text_cache.mjs`.
- `test_rawrarr_library_browser.mjs` — checks that Movies and TV use one responsive, searchable, filterable poster-card renderer with service-specific availability. Run with `node --test scripts/tests/test_rawrarr_library_browser.mjs`.
- `test_rawrarr_artwork_cache.rs` — checks disabled-cache file reads and disk fallback when Redis is unreachable; runs with the Rust suite or `cargo test artwork_cache` in the isolated test image.

- `test_rawrarr_coordinator_arr_links.mjs` — checks the coordinator's encrypted Radarr slug flow, direct metadata route, browser-only reachability gate, and year-cell icon wiring for Movies and TV. Run with `node --test scripts/tests/test_rawrarr_coordinator_arr_links.mjs`.
# Rawrarr tests

- `test_rawrarr_topology_visibility_browser.mjs` — Chromium checks that topology stays hidden for no devices and offline-only devices, appears online, and opens after a device reports online on a later poll. Run in the isolated Playwright test image.


- `test_internal_deploy_targets.py` — checks coordinator-only, node-only, shared/unknown runtime, documentation-only, and mixed-change target classification. Run `python3 scripts/tests/test_internal_deploy_targets.py`.
- `test_rawrarr_node_avatar.mjs` — verifies node-recipient validation, encoded relay route, no-store fetching, and deterministic SHA-256 avatar seed derivation; run with `node --test scripts/tests/test_rawrarr_node_avatar.mjs`.

- `secure_config::tests::node_identity_is_created_once_and_existing_identity_is_reused` and `invalid_existing_node_identity_is_not_replaced` — Subscriber startup age identity generation, persistence/reuse, separation from the user identity, file permissions, and fail-closed invalid-file behavior; run with `cargo test --locked --manifest-path rust-arr/Cargo.toml node_identity` in the Rust test environment.
- `secure_config::tests::node_signing_key_is_created_once_and_reused_separately_from_user_identity` and `invalid_existing_node_signing_key_is_not_replaced` — Subscriber ML-DSA-65 key creation, derivation stability across restarts, separation from the user's age identity, restrictive permissions, and fail-closed invalid-file behavior; run with `cargo test --locked --manifest-path rust-arr/Cargo.toml node_signing_key` in the Rust test environment.

- `test_rawrarr_admin_preview.mjs` — checks sample-only admin content, local-only port publishing, absence of a production data connection, and planned authentication before live reporting. Run with `node --test scripts/tests/test_rawrarr_admin_preview.mjs`.
- `test_rawrarr_subscriber_compose.mjs` — Validates subscriber fossil/boneyard Compose output, the pinned instance-root fingerprint, internal Tailscale coordinator URL and selected Headscale modes, secret-file auth, node-local Tailscale state, custom device names, and invalid network settings; run with `node --test scripts/tests/test_rawrarr_subscriber_compose.mjs`.
- `test_rawrarr_subscriber_onboarding_browser.mjs` — Chromium checks that first passkey registration enters the age/signing-key setup flow, backup gates and private-key boundaries, explicit optional Compose setup, supported node/network choices, safe Compose generation/download, and the Devices continuation; run in the isolated test image.
- `coordinator::approved_node_registry_tests` — verifies a valid empty approved-node registry lets a new coordinator start without pre-provisioned devices while malformed registries still fail; run with the Rust unit suite.
- `test_rawrarr_ai_chat.mjs` — checks browser-local movie filters, Movies-versus-TV disk usage shares, and candid missing-data responses. Run `node --test scripts/tests/test_rawrarr_ai_chat.mjs`.

- `test_rawrarr_landing_page.mjs` — Chromium checks for the signed-out landing page, header tagline, passkey sign-in links, accurate security/maturity disclosure, and mobile layout. Run with `node --test scripts/tests/test_rawrarr_landing_page.mjs` in the isolated test image.
- `test_share_storage_media.py` — Rejects unsafe media paths and plain HTTP remote APIs, requires the A/a filter for a root `/media` share, refuses overlapping folders, and checks send-only/receive-only pairing without auto-accept. Run `python3 scripts/tests/test_share_storage_media.py`.
- `test_add_music_sync_albums.py` — Checks exact album allow patterns, idempotency, and rejection of unsafe names. Run `python3 scripts/tests/test_add_music_sync_albums.py`.
- `test_hash_media_tree.py` — Checks path/content hashing and symlink refusal. Run `python3 scripts/tests/test_hash_media_tree.py`.
- `test_lock_syncthing_to_tailscale.py` — Checks that tailnet restrictions verify local identity and refuse to alter a Syncthing instance with unrelated peers. Run `python3 scripts/tests/test_lock_syncthing_to_tailscale.py`.

- `test_rawrarr_arr_summary.mjs` — checks that Radarr and Sonarr use the shared configurable four-card summary renderer. Run with `node --test scripts/tests/test_rawrarr_arr_summary.mjs`.

- `test_rawrarr_metadata_sizes.mjs` — checks the metadata API and Movies/TV cards report separate media-subtree sizes. Run with `node --test scripts/tests/test_rawrarr_metadata_sizes.mjs`.

- `test_rawrarr_sonarr_completion_ui.mjs` — confirms Movies and TV use the shared provider-aware scrape status/progress updater, and that the TV view hides scrape completion and successful-load messages after the Sonarr artwork cache is complete while retaining errors. Run with `node --test scripts/tests/test_rawrarr_sonarr_completion_ui.mjs`.

- `test_rawrarr_direct_route.mjs` — static regression checks for Tailscale direct metadata routing, the short-lived node-scoped session, separate node check-in and browser route labels, visible automatic selection, and coordinator fallback. Run with `node --test scripts/tests/test_rawrarr_direct_route.mjs`.
- `test_rawrarr_device_routes.mjs` — checks the shared all-device route-status helper for Direct, Relay, and Offline states, Redis cache reachability in measured node latency, cached probes, and explicit refresh. Run with `node --test scripts/tests/test_rawrarr_device_routes.mjs`.
- `test_rawrarr_device_routes.mjs` — checks the shared all-device route-status helper for Direct, Relay, and Offline states, Redis cache reachability in measured node latency, probe caching, and explicit refresh. Run with `node --test scripts/tests/test_rawrarr_device_routes.mjs`.

- `test_rawrarr_node_network_isolation.mjs` — checks that metadata/storage have no published host ports and share a Tailscale namespace with loopback-only listeners and relay targets. Requires Node.js; run `node --test scripts/tests/test_rawrarr_node_network_isolation.mjs`.
- `test_rawrarr_node_network_isolation.mjs` — also checks both metadata Compose variants use Valkey with bounded, non-persistent cache settings, health checks, local-only access, and the opt-in default disabled. Run `node --test scripts/tests/test_rawrarr_node_network_isolation.mjs`.

- `test_rawrarr_mini_export.mjs` — headless Chromium mini-export test using real post-quantum ciphertext from the `age` CLI; verifies browser decryption, HMAC filename matching, local ZIP contents, wrong-key handling, and that the private ID is never sent. Run in the isolated Playwright test image.

- `test_rawrarr_role_deployment.py` — disposable Docker coordinator/metadata/storage integration, real Chromium virtual-passkey flow for account-linked coordinator-origin credential addition (via `helpers/role_passkey_browser.mjs`), session scoping, logout, role mismatch and outage. Requires `rawrarr:roles-test`, PostgreSQL 17, Caddy, and the existing testing base image; dry-run by default, run with `--apply`.
- `test_rawrarr_storage_node.rs` — validates daily storage scan interval settings, bounded movie sample filtering, opaque cache keys, and standard MD5/SHA-1/SHA-256 output. Runs through the Rust test suite.
- `test_rawrarr_storage_scan_ui.mjs` — checks the Devices scan actions, bounded 1–2 GB movie sample, progress routes, and storage-only coordinator relay gate. Run `node --test scripts/tests/test_rawrarr_storage_scan_ui.mjs`.
- `test_rawrarr_coordinator_workspace.mjs` — coordinator passkey landing page, owner-scoped node list and relay, visible pending-node signing-dialog placement, roarlay/fossil/boneyard UI labels and Compose names, browser-profile age-key sharing and clearing, persisted age-ID rehydration and Movies unlock behavior, browser-side cluster config decryption and secret redaction, authenticated coordinator JPEG XL avatar delivery with PNG fallback from a public-age-ID digest, Tailscale SOCKS routing, delegated node sessions, accessible online/offline indicators, configured Radarr/Sonarr media-tab form visibility, and Sonarr series rows/summary rendering from the direct metadata API. Run `node --test scripts/tests/test_rawrarr_coordinator_workspace.mjs`; Rust avatar tests cover deterministic dinosaur traits, rarity, six scenes, spike/wings rules, and raster output formats.
- `test_rawrarr_node_enrollment.rs` — coordinator relay bounds, ephemeral P-256 encrypted-session validation, strict public-only enrollment metadata including the node signing proof, same-token/same-role legacy ID upgrade allowance, and origin/ID validation; included in the isolated Rust suite.
- `test_rawrarr_storage_node.rs` — validates local-time daily/weekly schedules, timezone/DST handling, bounded movie sample filtering, opaque cache keys, and standard MD5/SHA-1/SHA-256 output. Runs through the Rust test suite.
- `test_rawrarr_storage_scan_ui.mjs` — checks the Devices scan actions, bounded 1–2 GB movie sample, schedule controls/status, draft restoration, scheduled progress, weekly save payloads, progress routes, and storage-only coordinator relay gate. Run `node --test scripts/tests/test_rawrarr_storage_scan_ui.mjs`.

- `test_provision_node.py` — dry-run, protected token/verifier, duplicate preservation, and invalid scope tests; run `python3 scripts/tests/test_provision_node.py`.
- `test_rawrarr_node_roles.rs` and `test_rawrarr_node_auth.rs` — role/media boundaries and authentication proxy/transport allowlists; run `cargo test node_` in the isolated test image.

The tests below cover Rawrarr application behavior. Some require a disposable PostgreSQL database or Docker with OrbStack; read each test's prerequisites before running it.

- `test_rawrarr_artwork_source.mjs` — checks the Radarr movie API's shared-cache source classification and the Movies table artwork-source column. Requires Node.js; run `node --test scripts/tests/test_rawrarr_artwork_source.mjs`.

- `test_rawrarr_age_setup.mjs` — checks the JPEG XL logo source and PNG fallback, shortened age-identity display, private-key-only copying, identity-derived visuals, copy/backup acknowledgement gating, and browser-local age-ID persistence after successful vault unlock. Requires Node.js; run `node --test scripts/tests/test_rawrarr_age_setup.mjs`.
- `test_rawrarr_age_setup_browser.mjs` — runs the generated-ID and backup acknowledgement flow in headless Chromium, checking that the private key stays off the page, the copy action copies only the key, and Continue stays gated until acknowledgement. Requires the isolated test image built by `run_rawrarr_tests_in_docker.py`; run it inside the test container with `node --test /workspace/scripts/tests/test_rawrarr_age_setup_browser.mjs`.
- `test_rawrarr_docker_healthchecks.mjs` — checks Rawrarr/PostgreSQL readiness checks and unique naming for containers in `rawrarr-test-regression`. Requires Node.js; run `node --test scripts/tests/test_rawrarr_docker_healthchecks.mjs`.
- `test_rawrarr_brand_case.mjs` — checks that pages and runtime messages display the product name as lowercase. Requires Node.js; run `node --test scripts/tests/test_rawrarr_brand_case.mjs`.
- `test_rawrarr_passkey_ui.mjs` — checks browser-side decoding of WebAuthn API options and that first setup generates a dinosaur-themed account name without asking for one. Requires Node.js; run `node --test scripts/tests/test_rawrarr_passkey_ui.mjs`.
- `test_rawrarr_profile_color.mjs` — checks profile color and wing-word labels match the deterministic dinosaur avatar digest. Requires Node.js; run `node --test scripts/tests/test_rawrarr_profile_color.mjs`.
- `test_rawrarr_coordinator_passkey.mjs` — checks the signed-in metadata-to-coordinator one-time passkey handoff, origin-local ceremony page, account association, and RP configuration. Requires Node.js; run `node --test scripts/tests/test_rawrarr_coordinator_passkey.mjs`.
- `test_rawrarr_configuration.py` — checks encrypted configuration, secret redaction, scanner sidecars, and encrypted poster delivery against an isolated server and disposable PostgreSQL database. Requires Python 3, `psql`, `age`, `age-keygen`, a release binary, and a database ending in `_test`.
- `test_rawrarr_passkey.py` — checks passkey setup, challenge/session handling, and malformed credential rejection against an isolated PostgreSQL database. Requires Python 3, `psql`, a release binary, and a database ending in `_test`.
- `test_local_radarr.py` — checks safe extraction of the node-local Radarr config archive and summary output without Docker, Radarr, or secret/path output. Run `python3 scripts/tests/test_local_radarr.py`.
- `test_rawrarr_radarr_recovery.py` — checks encrypted-backup scheduling, UI-authenticated backup download boundaries, archive/path validation, local age decryption, and Radarr restore upload without Docker or a live Radarr instance. Run `python3 scripts/tests/test_rawrarr_radarr_recovery.py`.
- `test_rawrarr_syncthing_peers.py` — checks the process-based Syncthing peer flow on systems without Docker. Requires Python 3 and Syncthing.
- `test_rawrarr_syncthing_orbstack.py` — starts isolated web/storage Syncthing containers in the `rawrarr-test-regression` Docker group and verifies real folder and file transfer. Requires OrbStack's Docker context and Python 3; use `--keep` to leave the labeled test group running for inspection.
- `run_rawrarr_tests_in_docker.py` — creates each uniquely named `rawrarr-test-regression` runner from `rawrarr-testing-base:local`, copies the current source and tests into its copy-on-write layer, then runs Node and Rust tests offline with retained logs and a final summary naming failed cases. The base contains Chromium, pinned Playwright driver, external tools, and cached Rust dependencies, not project source; it is built only when missing unless `--rebuild-base` is supplied. Container and visible service names include a Unix timestamp encoded in lowercase base36. Run with `python3 scripts/tests/run_rawrarr_tests_in_docker.py`; add `--remove-after` to remove the finished container after displaying its logs. Requires OrbStack's Docker context.
- `run-tests-in-container.sh` — runs the Node.js, Playwright, and Rust unit suites in an isolated test runner. Invoked by `run_rawrarr_tests_in_docker.py`.

- `run_rawrarr_tests_in_docker.py` — the final container summary includes test start time, stop time, and elapsed seconds. Each invocation checks whether cleanup is due; at most once per hour it removes only stopped `tests-*` containers in the `rawrarr-test-regression` project that were created more than one hour ago. Preview candidates without deleting them with `python3 scripts/tests/run_rawrarr_tests_in_docker.py --cleanup-dry-run`.

- `test_rawrarr_navigation.mjs` — Chromium regression coverage for Movies/Configuration reload, in-page refresh, history, direct links, fallback, and Settings. Uses the existing isolated test image with Playwright and Chromium; run `node --test /workspace/scripts/tests/test_rawrarr_navigation.mjs` inside it.

- `test_rawrarr_movie_links.mjs` — Chromium checks for reachable, blocked, timed-out, missing-slug and reverse-proxy movie links, including poster/title clicks. Run in the existing isolated test image with Node.js, Playwright, and Chromium.

- `test_rawrarr_quality_profiles.rs` — Rust cache-first quality-profile tests: batch fetch, persistence, warm cache with offline upstream, preserved fields, changed assignments, and failure handling. Run `cargo test --locked --offline --manifest-path rust-arr/Cargo.toml quality_profiles` in the isolated testing image.

- `test_rawrarr_ai_chat.mjs` — local library answers for year, genre, and disk-size shares, plus split-frame OpenAI SSE parsing. Run `node --test scripts/tests/test_rawrarr_ai_chat.mjs`.

- `test_rawrarr_filename_secret.rs` — Rust tests for random key generation, validation, encrypted persistence, legacy migration, peer consistency, Radarr update/removal preservation, and plaintext-mirror exclusion. Included in the isolated Rust suite.
- `test_rawrarr_config_schema.rs` — Rust tests for legacy flat config migration to schema v1, service-array persistence including optional encrypted Radarr UI credentials, and unsupported-version rejection. Included in the isolated Rust suite.
- `test_rawrarr_hmac_metadata_filenames.rs` — checks domain-separated opaque movie filenames, byte-identical ciphertext migration, and decryption after migration. Included in the isolated Rust suite.

- `test_rawrarr_movie_responsiveness.mjs` — Chromium checks that pending Radarr probes do not block rows, refresh, or posters, and failed refreshes retain the previous list. Run in the existing isolated Playwright test image.

- `test_rawrarr_movie_identifiers.mjs` — Chromium checks for clean movie titles, separate TMDb IDs, missing-ID placeholders, column alignment, and omitted movie paths. Run in the isolated Playwright test image.

- `test_rawrarr_metadata_cache.rs` — vault-local ciphertext cache reuse, same-length replacement invalidation, missing files, identity isolation, and concurrent replacement safety. Run `cargo test metadata_cache`.

- `test_setup_local_syncthing.py` — Sanitized local-review bootstrap tests: preserve API key, idempotent folder setup, enable global discovery/public relays/NAT, reject remote peers and unrelated folders. Run `python3 scripts/tests/test_setup_local_syncthing.py`; no Docker required.

- `test_rawrarr_movie_snapshot.rs` — snapshot lifetime, empty libraries, shared client clones, replacement-client isolation, and direct Radarr poster records cached by TMDb ID. Run `cargo test movie_snapshot`. The responsiveness browser tests also cover immediate snapshot display and late-response ordering.

- `test_rawrarr_sonarr.rs`: Sonarr series normalization and legacy config compatibility; `cargo test sonarr`.
- `test_rawrarr_sonarr_metadata.rs`: TVDB-domain-separated HMAC names and age-encrypted Sonarr metadata/artwork storage; run `cargo test sonarr_metadata` in the isolated test image.
- `secure_config::tests::encrypted_config_and_artwork_are_shareable_age_ciphertexts` — encrypted 512 KiB artwork round-trip regression for large age subprocess pipe deadlocks; run `cargo test --locked --manifest-path rust-arr/Cargo.toml secure_config::tests::encrypted_config_and_artwork_are_shareable_age_ciphertexts -- --exact` in the isolated Rust test image.
- `test_rawrarr_sonarr_scrape.mjs`: metadata-node Sonarr backfill startup, cache-first scraping, shared one-record-at-a-time low-rate scheduling across Radarr and Sonarr, immediate save notification, six-hour repeat, allowlisted TVDB artwork fallback, and TV metadata storage boundaries; run `node --test scripts/tests/test_rawrarr_sonarr_scrape.mjs`.
- `test_rawrarr_arr_settings.rs`: encrypted cross-app save/clear isolation and filename-key preservation; `cargo test arr_settings` (age required).
- `test_rawrarr_sonarr_settings.mjs`: Chromium flow for entering a Sonarr URL/API key, saving, clearing the password field after save, reloading only the URL, and clearing the saved connection; run in the isolated Playwright test image.
- `test_rawrarr_coordinator_sonarr_settings.mjs`: Chromium flow for Radarr and Sonarr forms on Movies, TV, and Settings; verifies cached Movies and TV entries stay hidden without an age ID; saved URL/key-state confirmation without exposing API keys; age-ID persistence after unlock, offline Settings import with local recipient validation, invalid-ID rejection, reload restoration, sign-out and manual-clear cleanup; direct metadata save/read-back/clear; and optional Radarr UI credentials are cleared from the browser after save, never returned by read state, and bypass roarlay. Run in the isolated Playwright test image.
- `test_rawrarr_shared_library.mjs`: Chromium fixture test proving column changes propagate across Radarr and Sonarr, provider-specific values, and navigation; `node --test scripts/tests/test_rawrarr_shared_library.mjs` in the test image.
- `test_rawrarr_node_enrollment.rs`: strict pending-node payload, age-identity exclusion, ML-DSA proof fields, in-place legacy node upgrade constraint, encrypted-session P-256 key validation, and origin/ID validation; included in the isolated Rust suite.
- `test_rawrarr_node_approval_ui.mjs`: signed-in, account-scoped pending-node approval, rejection and deletion UI/API behavior, enrollment-only metadata pass-through, bounded bearer-token forwarding, display-name retention, and approved-node name synchronization; included in the isolated Node.js suite.
- `test_rawrarr_device_names.mjs` — Verifies device names stay in age config, browser encryption/decryption keeps names out of coordinator service storage, and the coordinator uses a dedicated Devices tab with inline name editing; run with `node --test scripts/tests/test_rawrarr_device_names.mjs`.

- `test_rawrarr_artwork_timing.mjs` — Per-request durations, lazy wait exclusion, missing timing and resource observer collection; run with Node.
- `test_rawrarr_artwork_timing_browser.mjs` — Chromium regression for scrolling to a deferred poster after a wait; run in the isolated Playwright image.
- `test_rawrarr_artwork_backfill.mjs` — Metadata-node background scraping, sequential requests, encrypted cache reuse, and Movies progress bar; run `node --test scripts/tests/test_rawrarr_artwork_backfill.mjs`.

- `test_coordinator_health.py` — Read-only live coordinator health-page and protected-auth checks; run `python3 scripts/tests/test_coordinator_health.py --base-url https://rawrarr.risk-mermaid.ts.net`. Requires Python 3.

- `test_rawrarr_device_signing_crypto.mjs` — checks owner-side ML-DSA-65 root creation, age-encrypted backup, folded-armor import and signing, root fingerprint matching, and tamper rejection; run with `node --test scripts/tests/test_rawrarr_device_signing_crypto.mjs`.
- Rust `shared_logs` tests verify signed compressed records, event tampering rejection, and offline node-certificate validation against an independently pinned instance-root fingerprint; run with `cargo test --bin rust-arr shared_logs::tests`.
`test_rawrarr_library_detail_link_layout.mjs` checks that Movies and TV display year and Arr detail links in the artwork/title column. Run with `node --test scripts/tests/test_rawrarr_library_detail_link_layout.mjs`.

`test_rawrarr_library_detail_link_layout.mjs` checks that Movies and TV display year and Arr detail links in the artwork/title column. Run with `node --test scripts/tests/test_rawrarr_library_detail_link_layout.mjs`.
- `test_ui_bundle.py` — Verifies the UI package allowlist/checksum, GitHub release and asset requests, Ed25519 signature acceptance/rejection, protected token propagation, installed verification metadata, repeat checks, ZIP path traversal rejection, and the 60-second minimum poll interval. Requires Python 3 and OpenSSL. Run `python3 scripts/tests/test_ui_bundle.py -v`.
- `test_rawrarr_coordinator_workspace.mjs` — Checks that Movies unlocking requires an age ID and that cached TV data is not rendered without one; run `node --test scripts/tests/test_rawrarr_coordinator_workspace.mjs`.
- `test_rawrarr_ui_signature_status.mjs` — Verifies the updater is coordinator-only, the authenticated Rust status uses installed signature metadata, and the profile badge exposes Signed/rejected states. Run `node --test scripts/tests/test_rawrarr_ui_signature_status.mjs`.
