# Fix Radarr and Sonarr poster delivery

## Objective

Restore the poster path for the coordinator's Movies and TV views and verify that the MacBook metadata node scrapes artwork into the intended shared folder.

## Initial state

Main was at `16e2cb0`. The primary checkout had pre-existing edits; work was isolated on `codex/fix-arr-posters`. Direct Radarr library loading returned 1,945 entries, but individual poster requests failed. The direct poster handler fetched the whole Radarr library for every image request, despite the live-library route already fetching it. Earlier observations showed the selected metadata node's `/shared/metadata/tv/artwork` empty, while a separate host `shared-data` tree contained encrypted files and had a different age recipient from the active node volume. Those locations cannot safely be merged without resolving their ownership and keys.

Related procedures: [Sonarr metadata backfill](2026-10-18-automatic-sonarr-metadata-backfill.md), [TV artwork display](2026-10-18-show-sonarr-series-artwork.md), [Radarr artwork](2026-10-18-radarr-movie-artwork.md).

## Actions

- Cached raw Radarr movie records by TMDb ID alongside the existing UI snapshot when the direct library route loads. Poster requests now use the cached record and retain the existing fetch fallback when no library snapshot exists.
- Added a unit regression for poster lookup from the direct library cache and extended the coordinator route regression.
- Preserved the existing host `shared-data` and node `/shared` data because their age recipients differ; no storage mount or ciphertext was changed.
- Updated the Mac mini `rawrarr-internal-mini` roarlay image from the task branch and kept its Tailscale Serve route pointing to the coordinator on port 8080.
- Updated the MacBook `rawrarr-node-internal` fossil image with Sonarr scrape fixes: failed episode-list requests no longer prevent series metadata and artwork writes, and worker errors identify the failing stage.
- Started the Sonarr backfill in the authenticated app. The MacBook node's named `/shared` volume now contains encrypted TV text and artwork (54 artwork files observed during this run). This volume is distinct from the host `shared-data` tree; no ciphertext was copied between locations.
- Movies still waits for Radarr's direct route. Live checks from the MacBook node had Radarr unreachable, so Radarr poster rendering remains unverified and the task branch has not been merged.

## Verification

- `node --test scripts/tests/test_rawrarr_coordinator_arr_links.mjs` passes (2 tests).
- Focused Docker Rust tests `cargo test --locked --manifest-path rust-arr/Cargo.toml sonarr::tests` pass (7 tests).
- `node --test scripts/tests/test_rawrarr_sonarr_scrape.mjs` passes (3 tests).
- `git diff --check` passes.
- MacBook fossil container is healthy and its loopback `/health` responds; TV UI backfill reached 54 / 227 artwork files saved.
- Mac mini roarlay is healthy; `https://rawrarr.risk-mermaid.ts.net/health` responds and Tailscale Serve reports `/` proxying to `http://127.0.0.1:8080`.
- Browser Movies view remains at “Connecting to Radarr through the node direct route…” with no library entries. Radarr reachability and visible movie poster rendering remain unverified.
- `node --test scripts/tests/test_rawrarr_coordinator_arr_links.mjs` passes (2 tests); `git diff --check` passes.
- Host `cargo test` cannot start because the installed LLVM toolchain requires the missing `/usr/local/opt/z3/lib/libz3.5.1.dylib`.

## Recovery

Revert the scoped code/test changes to restore per-poster Radarr list requests. No shared metadata, encrypted files, or service configuration has been changed by this task.

## Remaining work

Restore Radarr reachability, then verify movie posters in the authenticated Movies view. Verify TV poster fetches in the consumer view after backfill finishes. Only then commit the remaining task-owned changes and merge into `main`. Do not reconcile the separate host `shared-data` tree with the MacBook node's named volume until its recipient and synchronization ownership are established.
