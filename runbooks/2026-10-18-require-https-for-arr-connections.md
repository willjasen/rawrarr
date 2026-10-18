# Require HTTPS for Arr connections

## Objective and initial state

Require encrypted HTTPS transport whenever Rawrarr connects to Radarr, Sonarr, and other Arr services. The shared Arr client previously accepted both HTTP and HTTPS.

## Actions

- Updated shared Arr URL validation to accept only HTTPS, which applies to Radarr and Sonarr.
- Added a Rust unit test that accepts HTTPS Radarr/Sonarr URLs and rejects HTTP and FTP.

## Verification

- `python3 scripts/tests/run_rawrarr_tests_in_docker.py`: all 32 Rust tests and the Playwright test passed. The Node suite had one unrelated failure, `movie API preserves Radarr titleSlug for detail links`.
- The Python configuration integration test is not part of that runner and was not verified.
- Direct host `cargo test` was unavailable because the host Rust toolchain aborts while loading a missing Homebrew `libz3.5.1.dylib`.

## Final state and remaining work

The shared ArrClient rejects non-HTTPS endpoints; Radarr and Sonarr are covered by the same constructor test. Integration-level HTTPS connection success remains unverified.
