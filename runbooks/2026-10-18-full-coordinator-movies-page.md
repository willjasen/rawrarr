# Full Movies page through coordinator ciphertext relay

## Objective and initial state

Restore the normal full Movies table at the coordinator while preserving the ciphertext-only trust boundary. The coordinator workspace currently decrypts only three cached movie records and renders them as cards. The standard metadata-node page obtains complete Radarr inventory directly, which must not pass plaintext through the coordinator.

## Actions

- Added a metadata-node endpoint that obtains the full Radarr inventory and encrypts it to a one-request browser session. The browser creates an ephemeral P-256 ECDH key pair, the metadata node derives an AES-256-GCM response key, and the coordinator only relays opaque bytes. The browser session is separate from age; age continues to protect shared metadata/export files.
- Kept the coordinator relay allowlist restricted to existing ciphertext export GET endpoints plus exactly `POST api/metadata/session/library` with a valid 65-byte uncompressed P-256 public key. The coordinator verifies account ownership and node role before forwarding; arbitrary methods/paths and malformed key bodies remain rejected.
- Replaced the three-card view with the usual Radarr summary metrics and full movie table, including lazy-loaded encrypted artwork. The browser-held age private ID is no longer required to load the library.
- A deployment using an incomplete Compose file set recreated services without their existing mounts and left metadata restarting. Recovered the coordinator in its existing Tailscale namespace and restored metadata as `rawrarr-local-metadata-recovery` with the existing named volume, shared-data mount, and token file. The coordinator and tailnet routing remained healthy. Do not remove this recovery service until the normal metadata Compose service has been restored safely.

## Verification

- Focused coordinator workspace Node tests: 5 passed. JavaScript syntax and `git diff --check` passed.
- Full Rust unit suite in the isolated container: 37 passed. The host Rust toolchain cannot run because its LLVM dylib references a missing Z3 library.
- The full isolated runner had 8 pre-existing unrelated Node failures (age setup VM fixture, artwork timing fixtures, naming, title slug, and size formatting); its Playwright test passed.
- Built the P-256/AES-GCM implementation successfully in Docker. The current running coordinator health and HTTPS `/health` checks return HTTP 200, and `tailscale serve status --json` confirms HTTPS 443 proxies to `http://127.0.0.1:8080`.
- Metadata recovery health on `http://127.0.0.1:7297/health` returns HTTP 200. The updated coordinator workspace module and metadata-export bundle are served successfully and expose `loadRadarrLibrary`.
- The relay allowlist test for valid and invalid ephemeral-key POSTs passed. The updated coordinator rebuilt in its Tailscale namespace; loopback metadata, coordinator, and HTTPS coordinator health all return HTTP 200, and Serve still targets `http://127.0.0.1:8080`.
- End-to-end browser decryption against a live Radarr request is not yet verified.

## Recovery and remaining work

The updated coordinator is running in its existing Tailscale namespace. Local metadata recovery serves port 7297 using the existing named volume, shared-data mount, and token file. The standard Compose metadata container remains stopped with restart disabled; keep the recovery container until the normal metadata service is restored safely. Deployment locks remain held for this recovery.
