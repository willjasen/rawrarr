# Create age-encrypted sidecars during library scans

## Objective and initial state

Make a Rawrarr storage/scanner node write JSON metadata for discovered media into the Syncthing-shared Rawrarr folder. Existing scans only returned filename and size inventory; Rawrarr already used age-encrypted `.aged` files for shared config and artwork. See [local library scanning](2026-10-17-local-library-scan.md), [encrypted artwork](2026-10-17-rawrarr-encrypted-artwork.md), and [device architecture](../docs/device-architecture.md).

## Actions

- Each successful scan now writes one atomic age-encrypted `<sha256(device_id + library_name + relative_path)>.json.aged` record under `metadata/library/` in `RAWARR_SHARED_DATA_DIR`.
- The encrypted JSON records schema version, stable device ID, library name, scan time, relative media path, kind, file size, and modification time. Absolute media paths and media bytes are not included.
- The scanner gets an explicit `RAWARR_DEVICE_ID` or creates a UUID once under node-local `RAWARR_DATA_DIR/device-id`, so identical relative paths on different storage nodes do not collide.
- On web-node age unlock, the public `age1pq1…` recipient is written to shared `age-recipient`; a storage-only scanner uses that public key and never receives the private identity. Existing conflicting recipient files are rejected.
- Added `storage-scan-once`, which scans on a storage node without web server, PostgreSQL, or private age identity. The web API scanner still requires an unlocked post-quantum age identity and returns HTTP 423 otherwise. Per-file encryption failures are counted in the scan result and shown in the UI.
- Added test coverage for sidecar ciphertext, decryption, traversal rejection, and the authenticated scan API. The Rust implementation keeps `unsafe` forbidden.
- Reviewed the sibling Gamarr, Tephramesh, and Synkpieces project docs for related Arr metadata, age/Syncthing trust boundaries, and hosted Stripe patterns. They remain unchanged.

## Verification

- Rust 1.85.1 `cargo fmt`, locked `cargo test` (5 passed), and locked release build passed. The age sidecar round-trip test verified `.json.aged` ciphertext, decryption, traversal rejection, and public-recipient conflict rejection.
- A disposable isolated smoke test ran the storage-only command with no PostgreSQL or private identity file, then decrypted the resulting sidecar and verified a repeated scan overwrote the same stable path. Node UI tests passed (3); `git diff --check` passed.
- The authenticated HTTP integration test was updated for sidecar and locked-vault behavior, but not executed because `RAWARR_TEST_DATABASE_URL` is unset.
- The system Rust launcher failed because its LLVM dependency cannot find `libz3.5.1.dylib`; using the installed Rust 1.85.1 toolchain succeeded.
- Rebuilt and restarted only the Rawrarr service in OrbStack under its existing Compose project; PostgreSQL remained healthy. The first Compose attempt inferred a new project name from the symlinked checkout, so its duplicate containers/network were removed before retrying with the existing project name.
- Verified `http://127.0.0.1:8084/health` returned `ok`; `/` and `/auth` both returned HTTP 200 HTML. No scan was started against the mounted library.

## Final state and remaining work

Storage nodes can now scan through a one-shot mode and write age-encrypted JSON sidecars directly into shared Rawrarr data without running the web/database process. The web-to-storage-node remote scan command/pairing protocol is still future work. The PostgreSQL-backed HTTP integration remains to be run with a disposable `_test` database.
