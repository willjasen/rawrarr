# Keep a local Radarr settings mirror beside encrypted shared config

## Objective and initial state

Let local tools inspect the configured Radarr connection without passkey or age handling while keeping production settings encrypted. Rawrarr already stored settings in `/shared/config.aged`; local `/data` held only a device ID. The local Compose service is loopback-bound at `127.0.0.1:7297`. Related context: [encrypted Radarr settings](2026-10-17-rawrarr-age-encrypted-config.md), [local data domains](../docs/data-domains.md), and [local service port](2026-10-18-rawrarr-local-port-7297.md).

## Actions

- Added an opt-in node-local `radarr-config.json` mirror with a `0700` data directory and `0600` file. The local Compose environment enables it; application default is disabled.
- Settings saved through Rawrarr update the encrypted shared file and, in local mode, the plaintext mirror. Unlocking reads the shared ciphertext as authoritative and refreshes the mirror. Local startup can load the mirror without an age identity.
- Startup in production mode removes any stale plaintext mirror. Shared configuration remains age encrypted.
- Updated data-domain and user documentation.

## Verification

- The Rust unit suite passed in the isolated test container (10 tests), including encrypted/local mirror contents and mode checks.
- The Node and Playwright suites passed (7 and 1 tests). The focused local Radarr helper tests passed (2 tests); `docker compose config --quiet` and `git diff --check` passed.
- The local service remained healthy on `127.0.0.1:7297`. Chrome showed Radarr 6.4.4.10685, 1,941 tracked movies, 1,902 with files, and about 8.0 TB reported usage.
- The configuration integration test was not run because the isolated test runner skips PostgreSQL-backed tests. Direct local clipboard transfer was interrupted before writing a plaintext config file; `/data/radarr-config.json` remains absent. The live service was not restarted because it currently has no saved local age identity and an unverified restart could lose its active Radarr connection.
- Host `cargo fmt --check` could not start because the installed LLVM is missing `libz3.5.1.dylib`; the isolated test image does not include `rustfmt`.

## Final state and remaining work

The code now mirrors successful Rawrarr config saves into private local `/data` when `RAWARR_LOCAL_PLAINTEXT_CONFIG=true`. Production defaults to false and removes any stale local copy at startup. The current local container has not been rebuilt, so it does not yet create or use this mirror. After a safe rebuild and one successful Rawrarr config save/unlock, `python3 scripts/local_radarr.py` can read Radarr status and library totals without passkey or age handling. The encrypted `/shared/config.aged` file remains authoritative when unlocked.
