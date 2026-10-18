# Store Rawrarr metadata artwork with age encryption

## Objective and initial state

Store movie poster art returned by metadata lookup as age-encrypted files using the `.aged` extension. Existing configuration encryption uses the same unlocked PQ age identity and `age` CLI; see [encrypted config](2026-10-17-rawrarr-age-encrypted-config.md). Metadata lookup currently returns text candidates only; see [movie metadata matching](2026-10-17-movie-metadata-matching.md).

## Actions

- Added an encrypted artwork cache at `data/metadata/movies/<tmdb-id>.aged`. The encrypted plaintext contains an allowed image MIME type and image bytes. Writes are atomic and use private file/directory permissions.
- Movie lookup accepts poster artwork only from HTTPS `image.tmdb.org` paths under `/t/p/`, disables redirects, allows JPEG/PNG/WebP only, and caps images at 8 MiB.
- Candidate results expose only a local poster route when artwork was cached. The route decrypts artwork only while the age vault is unlocked and returns `private, no-store` responses.
- Added a Rust age encrypt/decrypt round-trip test and extended `scripts/tests/test_rawrarr_configuration.py` to verify `.aged` delivery, response MIME type, and the locked-vault response.

## Verification

- `cargo fmt --check`, locked `cargo check`, locked `cargo test` (two tests), and locked release build passed with Rust 1.85. Tests cover `.aged` encryption/decryption round-trip and strict TMDb image URL validation.
- `python3 scripts/tests/test_rawrarr_configuration.py` passed. It confirmed ciphertext storage, no poster plaintext in the `.aged` file, local poster delivery while unlocked, HTTP 423 while locked, config-key redaction, and cleanup of isolated test data.
- `sh scripts/tests/test_run_media_sidecar_throttled.sh` and `git diff --check` passed.
- A fresh build is serving on `127.0.0.1:8083` with an empty temporary data directory. `/health` returned 200 and the served UI includes the `.aged` cache description. The existing `8080` and `8082` processes were left untouched. No live Radarr/TMDb request or Docker build was performed.

## Final state and remaining work

- Poster image storage and retrieval use `.aged` ciphertext, and the UI displays locally served posters from age-encrypted cache files. Open `http://127.0.0.1:8083/` to inspect this build. Full live metadata fetch behavior and Docker image build remain unverified.
