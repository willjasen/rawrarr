# Persist Radarr settings in age-encrypted config

## Objective and initial state

Persist the Radarr URL and API key from the Rawrarr UI in an age-encrypted config file, apply changes immediately, and load saved settings after restart when the age identity is available. Initial state: UI settings are held only in process memory. Related references: [Radarr UI settings](2026-10-17-rawrarr-radarr-ui-config.md) and [no-restart settings](2026-10-17-apply-radarr-settings-without-restart.md).

## Actions

- Added PQ age identity generation/import and unlock endpoints. The private identity is held by Rawrarr in memory or read from `RAWARR_AGE_IDENTITY_FILE`; it is never written into the config file.
- Radarr URL and API key are encrypted with the age v1.3.2 CLI to `config.age`, written with mode `0600`, and applied to the running client without restart. Clear removes both runtime settings and encrypted storage.
- After unlock, the UI can update the Radarr URL without revealing or requiring re-entry of the saved API key. Sensitive request, identity, and serialized config buffers use zeroize where owned by the Rust app.
- Added a Settings UI for creating/importing an identity, an age CLI build stage, durable Docker volume, optional Compose secret override, and setup documentation.
- Added `/data/` to the repository ignore rules and indexed this runbook in [the runbook overview](README.md).
- The workstation's default `rustc` launcher was linked against a missing LLVM/Z3 library. Using the installed Rust 1.85 toolchain directly through `PATH` resolved local compilation.

## Verification

- Rust 1.85 `cargo fmt --check`, locked `cargo check`, and locked release build passed. `cargo clippy` could not start because the workstation's `clippy-driver` needs the missing `libz3.5.1.dylib`.
- Manual HTTP flow passed: generated a PQ identity, imported it, saved Radarr settings, confirmed the ciphertext contains no test API key and has mode `0600`, restarted with the identity mounted, authenticated against a local Radarr mock, updated the URL with the key omitted, and cleared the encrypted file.
- `go install filippo.io/age/cmd/...@v1.3.2` produced both age commands; Compose configuration parsed successfully. The Docker daemon is not running, so an image build was unavailable.
- The updated dashboard is running on `127.0.0.1:8081` and was inspected in the browser. The existing `127.0.0.1:8080` instance has a configured Radarr connection but predates encrypted persistence; it was left running to preserve its in-memory API key.

## Final state and remaining work

- UI-entered Radarr settings now survive restart as age-encrypted config. The web UI remains loopback-bound; use the mounted identity secret for unattended container unlock. To migrate the existing `8080` connection, unlock the new `8081` instance and enter the Radarr key there so Rawrarr can persist it encrypted. Verify the production Docker build once a Docker daemon is available.
