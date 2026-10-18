# Version the shared config and group services

## Objective

Give the age-encrypted cluster config a documented schema and represent Radarr under a typed `services` array entry.

## Initial state

The persisted config used flat `radarr_url` / `radarr_api_key` fields and a separate `sonarr` object. Browser-side device-name edits rewrote the decrypted document directly. Related history: [encrypted config](2026-10-17-rawrarr-age-encrypted-config.md), [configuration view](2026-10-17-rawrarr-configuration-page.md), and [redacted cluster config](2026-10-18-show-redacted-cluster-config.md).

## Changes

- Added the version 1 JSON Schema at `docs/shared-config-v1.schema.json`.
- Rust serialization writes `schema_version: 1`, `device_names`, `services`, and `metadata_filename_key`; Radarr and Sonarr are typed service entries with URL and API key fields.
- Rust deserialization continues to accept the legacy flat fields and migrates them to v1 on unlock. Unsupported future schema versions are rejected; unknown service types are retained.
- Browser-side device-name writes upgrade legacy config to v1 and keep the metadata filename key present. Browser cleanup now recursively clears decrypted config strings, including service API keys.
- The coordinator browser presents legacy documents in the v1 shape without rewriting the shared ciphertext merely to view it.
- Rebuilt and restarted only `roarlay` in `rawrarr-local` using the active coordinator and MacBook Tailscale Compose overrides. Roarlay is healthy; HTTPS health identifies roarlay and Serve still proxies `/` to `127.0.0.1:8080`.
- Metadata and storage containers were not deployed. The v1 Rust persistence migration is ready in source; existing subscriber ciphertext is displayed in v1 shape and will be rewritten when an updated subscriber node unlocks it or a browser name edit saves it.

## Verification

- Isolated Docker Rust suite: 43 passed.
- Focused device-name Node checks: 3 passed. In the coordinator workspace suite, 8 checks passed and one existing series-heading assertion failed because the current workspace page's heading does not match that test's expected string.
- The deployed `/assets/metadata-export.js` SHA-256 matches the generated local bundle.
- JSON Schema parses as JSON and `git diff --check` passes.
- The full container Node suite reported 10 existing failures in unrelated age-setup, artwork timing, naming, coordinator/direct-route, movie-link, and network-isolation tests. Some failures reflect files absent from the test container's copied workspace; focused affected checks passed.
- Host `cargo test` could not start because the installed LLVM requires missing `libz3.5.1.dylib`; the isolated Rust suite compiled and ran successfully.

## Recovery

Revert the task-owned schema, serialization/deserialization migration, browser config normalization/cleanup, focused schema tests, documentation index changes, and this runbook. Legacy config remains readable until the migration is reverted.
