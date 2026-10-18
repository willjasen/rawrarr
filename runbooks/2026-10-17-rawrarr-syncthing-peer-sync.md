# Rawrarr Syncthing peer sync

## Objective and initial state

Add a Rawrarr integration that configures a web node and a storage node to share a folder using Syncthing, then verify a real folder and file transfer. The repository currently describes remote scanner/device synchronization as future work; see [device architecture](../docs/device-architecture.md) and [device model](2026-10-17-rawrarr-device-model.md).

## Implementation direction

Syncthing's Block Exchange Protocol (BEP) handles authenticated peer sync over TLS. Rawrarr will manage supported Syncthing nodes through their authenticated REST API rather than reimplementing BEP. The REST API key is sent only in an `X-API-Key` header and never logged or rendered. The web/storage test nodes use isolated config, state, and folder directories.

## Verification and remaining work

Record Rust checks and peer-to-peer folder/file verification here. The default test cleans up isolated state; `--keep` is available when the user wants the test nodes visible in OrbStack. If local Docker is unavailable, use isolated Syncthing processes from an official release and document that execution path.

## Implementation and verification

- Added a memory-safe Rust REST client in `rust-arr/src/syncthing.rs`; crate-level `#![forbid(unsafe_code)]` remains in force. It loads the local API URL/key from environment settings, can read a key from a mounted file, and uses authenticated requests to add both peer identities and set the web folder to send-only and storage folder to receive-only.
- Added authenticated Rawrarr endpoints for local Syncthing status and configuring a remote share. Added a Devices panel form; the remote API key is cleared from the UI after success and is not persisted by Rawrarr.
- Added the OrbStack integration script `scripts/tests/test_rawrarr_syncthing_orbstack.py`; it makes a temporary Docker network, starts two Syncthing v2.1.5 containers, configures the pair through REST, checks sync state and SHA-256 of a nested file, and removes containers/network/temp files on completion.
- OrbStack integration command: `python3 scripts/tests/test_rawrarr_syncthing_orbstack.py`.
- Result: PASS — `syncthing/syncthing:2.1.5` ran on Docker context `orbstack`; the independent web and storage containers connected, their folder roles were configured, and `metadata/movies/sample-metadata.json` arrived with matching SHA-256. A second run used `--keep`; the validated containers and isolated data remain available in OrbStack for inspection.
- Workstation Homebrew `rustc`/`rustfmt` abort because `libz3.5.1.dylib` is missing. In OrbStack, `cargo fmt` completed and `cargo test --locked syncthing::tests` passed (2 tests). The full Rust suite passed (5 tests) after supplying `age` and `age-keygen` built from the same pinned Go module used by the Rawrarr Dockerfile.

## Local OrbStack application

- Started the full `rawrarr-local` Compose project with the app and PostgreSQL on Docker context `orbstack`; selected host port 8083 because a pre-existing native Rawrarr process already occupied 8082 and was preserved.
- The PostgreSQL password is generated once and stored in an owner-only file outside the repository at `~/Library/Application Support/Rawrarr/orbstack-postgres-password`; Compose receives its path through `RAWARR_POSTGRES_PASSWORD_FILE`.
- The existing native Rawrarr processes occupied 8082 and 8083; both were preserved. Local WebAuthn origin is configured for `http://localhost:8084`.
- Fixed two Docker runtime packaging issues found during startup: included `migrations/` in the Rust build stage for `sqlx::migrate!` and installed `libssl3`/CA certificates in the runtime image.
- Final verification: both `rawrarr-local-rawrarr-1` and `rawrarr-local-postgres-1` are running in OrbStack, PostgreSQL reports healthy, `/health` on port 8084 returns HTTP 200 `ok`, `/` redirects to `/auth`, `/auth` returns the passkey onboarding page, the logo and favicon endpoints return HTTP 200, and the Docker UI is open in the in-app browser at `http://localhost:8084/auth`.

## Scope notes

Rawrarr manages Syncthing configuration through its REST API; Syncthing implements and transports its own encrypted peer protocol. The local Rawrarr API key must be provided through `RAWARR_SYNCTHING_API_KEY` or a mounted key file (`RAWARR_SYNCTHING_API_KEY_FILE`). The peer form uses the remote node's API key transiently. Ensure remote Syncthing API access is protected by a trusted network or HTTPS. This work configures folder sync; it does not encrypt arbitrary folder contents itself. Data intended to be age encrypted must be written as age ciphertext before Syncthing syncs it.
