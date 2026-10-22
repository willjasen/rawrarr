# Throttle Arr metadata scraping

## Objective

Prevent Radarr and Sonarr metadata backfills from querying the Arr services concurrently, and pace work for the MacBook metadata node.

## Initial state

Both backfill loops run independently on the MacBook metadata node. Each record scrape uses a shared mutex, but their initial library requests can overlap and the lock does not cover the full record operation. Existing poster-fix work is in progress in the same checkout; unrelated edits are preserved.

## Actions

- Extend the shared scrape mutex around each movie or series record operation, including metadata and artwork retrieval, so Radarr and Sonarr backfills serialize against each other and on-demand Radarr poster retrieval.
- Serialize each Arr library/profile listing through the same mutex; Sonarr loads its series list and quality profiles in sequence.
- Add a two-second delay between records in both backfills to reduce request rate.
- Preserve the existing per-service listing and cache-first behavior.
- Deployed the worker build to the MacBook node and refreshed the Mac mini coordinator source. The mini source bundle was missing migration 7; added the tracked migration and rebuilt from a clean native ARM build without changing its database volume.

## Verification

- `node --test scripts/tests/test_rawrarr_sonarr_scrape.mjs` passes (4 tests), including shared lock and two-second pacing assertions.
- Docker Rust tests `cargo test --locked --manifest-path rust-arr/Cargo.toml sonarr::tests` pass (7 tests).
- `node --test scripts/tests/test_rawrarr_coordinator_arr_links.mjs` passes (2 tests); `git diff --check` passes.
- MacBook fossil is healthy with zero restarts after deployment.
- Mac mini roarlay is healthy with zero restarts; the public HTTPS health check succeeds, and Tailscale Serve still proxies `/` to `http://127.0.0.1:8080`.
- The authenticated Movies view still reports `Radarr movie library request failed`; the reduced scrape rate does not restore Radarr reachability by itself.

## Recovery

Restore the previous record-scrape mutex scope and 250 ms interval. No shared metadata or service configuration should be modified by the code rollback.

## Remaining work

No deployment or health-check work remains. Investigate Radarr availability separately; do not infer that a slower scrape restores its network service.
