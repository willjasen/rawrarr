# Run Rust Arr locally

## Objective and initial state

Compile and start the Rust Arr application as a local web server, then verify its HTTP routes. The code is committed on `main`; system Rust currently aborts because LLVM expects an unavailable Z3 dylib. Docker daemon status is also currently unavailable.

## Actions

Pending.

## Verification

Pending.

## Final state and remaining work

Pending.

## Actions

- Installed isolated Rust 1.85.1 with rustup and put its binaries first in `PATH` for Cargo. This avoids the broken Homebrew `rustc`/LLVM/Z3 linkage without changing Homebrew packages.
- Formatted the source and built the release binary with `--locked`.
- Started the service on `127.0.0.1:8080` in a persistent foreground command session. No Radarr credentials were provided.

## Verification

- `cargo fmt --check` passed after formatting.
- `cargo build --release --locked` passed with Rust 1.85.1.
- `GET http://127.0.0.1:8080/health` returned `200` and `ok`.
- `GET http://127.0.0.1:8080/api/radarr/status` returned the expected `503` JSON response because Radarr is not configured.
- The live server is bound only to loopback. No live Radarr request was made.

## Final state and remaining work

The local Rust Arr web server is running on port 8080 in command session 20101. Open `http://127.0.0.1:8080/health`; stop the process with Ctrl-C in that session. Build artifacts are ignored under `rust-arr/target/`. Radarr status requires `RADARR_URL` and `RADARR_API_KEY` at process startup.
