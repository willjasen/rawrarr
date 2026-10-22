# Fix intermittent Sonarr library timeouts

## Objective and initial state

Resolve intermittent `Sonarr library request failed` errors in the authenticated TV view. Existing unrelated coordinator edits in the shared checkout are preserved. Fossil logs showed `/api/sonarr/series` returning HTTP 502 at 5,013–5,015 ms while Sonarr metadata/artwork scraping was active; a manual refresh later loaded 229 series. The shared Arr HTTP client timeout was five seconds.

## Actions

- Increased the shared Arr upstream request timeout to 30 seconds. Sonarr's series and quality-profile requests now have time to finish under metadata-node load; Radarr uses the same shared client.
- No credentials or vault contents were read.

## Verification

- `docker build -t rawrarr:local ./rust-arr` succeeded. The host Rust tests could not start because the installed LLVM lacks `libz3.5.1.dylib`; the isolated all-project runner reported 19 unrelated/stale Node failures and its Rust phase could not reach crates.io.
- Recreated only `rawrarr-node-internal` service `fossil` with `--no-deps`; the existing data and shared volumes and Tailscale sidecar were preserved. Container health is `healthy`, and `/health` returned HTTP 200.
- Refreshed the authenticated TV page. It reports `Loaded 229 series from Sonarr`, 13,827 episodes, and 10.1 TB. No library request error is visible.

## Recovery

Restore the `ArrClient` timeout in `rust-arr/src/main.rs` to five seconds and rebuild fossil if the longer timeout causes a confirmed issue.

## Remaining work

The broader isolated suite remains limited by pre-existing failures and unavailable crates.io access; the live TV request and fossil health checks succeeded.
