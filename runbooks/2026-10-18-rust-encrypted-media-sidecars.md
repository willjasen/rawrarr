# Rust encrypted media sidecars

## Objective and initial state

Extend automatic storage scans with file-derived hashes and ffprobe metadata. Store encrypted records in the shared metadata tree, never beside media. The existing scan wrote encrypted inventory records under `/shared/metadata/library/text/`; the referenced Python workflow supplied hashes and ffprobe details but used adjacent plaintext JSON.

## Implementation

- Storage scans calculate MD5, SHA-1, and SHA-256 in one streaming read and capture ffprobe `format` and `streams` JSON.
- Preserve the existing record envelope and opaque filename derived from device, library, and relative path. Age encryption continues to use the shared public recipient and stores `.json.aged` records under `/shared/metadata/library/text/`.
- The runtime image installs ffmpeg to provide `ffprobe`. `RAWARR_FFPROBE` can select a compatible executable for fixtures or custom installations.
- The one-shot scan reports per-file failures in the existing `sidecarErrors` count and never writes sidecars beside the media.

## Verification

- Focused Rust storage-node tests passed (4/4), including standard MD5, SHA-1, and SHA-256 vectors.
- A one-shot storage CLI fixture produced one decryptable `.json.aged` record under `shared/metadata/library/text/`; decrypted data contained all three hashes and the stub ffprobe format/stream records. The fixture confirmed no JSON was created beside media.
- Python integration fixture syntax compilation passed; the full configuration integration script was not run because it requires a disposable PostgreSQL `_test` database.
- `git diff --check` passed for the task changes. Formatting tools are unavailable in the host toolchain (LLVM cannot load its missing Z3 library), and the test image does not include `cargo-fmt`.

## Recovery and remaining work

Rebuild the storage node image to include `ffprobe`. Deployment to the Mini and a full live-media scan were not performed. Verify the resulting ciphertext in its synchronized shared folder after deployment.
