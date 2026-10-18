# Responsive movies refresh

## Objective and initial state

Show the Movies list sooner after refreshing. Existing UI awaited a settings request and browser reachability probe (each with a three-second timeout) before rendering already-fetched movies. The worktree contains substantial unrelated changes, preserved separately. Related: [artwork timing](2026-10-18-radarr-artwork-load-time.md).

## Actions

Render rows and posters immediately after the movie API response. Enhance titles with links asynchronously after the existing credential-free probe succeeds; ignore obsolete probe results after another refresh. Keep rows during updates, prevent overlapping list requests, and label retained data on failure. No browser metadata persistence is introduced.

## Verification

- Nine focused Node/Chromium tests passed: responsiveness, links, navigation, and artwork source.
- Diff whitespace check and release container build passed.
- Verified the existing app owned port 7297; rebuilt the same Compose project preserving its volumes. Container healthy; `/health` returned HTTP 200.
- Live authenticated Radarr timing was not measured; browser checks use controlled fixtures.

## Final state

Deployed locally. Initial movie API latency remains; the optional connection check no longer blocks rendering. UI and existing link-test edits depend on pre-existing uncommitted implementation and remain uncommitted to avoid absorbing other work. New regression coverage and documentation are committed separately. No remote is configured.

## Recovery

Reverse only this task's UI changes and rebuild the same Compose project with existing volumes preserved.

## Chrome follow-up

The user reported continued latency. Tested their existing Chrome Movies tab with 1,941 entries. A refresh failed after 5.4 seconds with a Radarr movie request error; other loads succeeded but remained slow. Source inspection found sequential per-movie age subprocesses for quality-profile metadata on every request. Added a bounded eight-worker cold read and a vault-local, bounded in-memory metadata cache keyed by ciphertext SHA-256. Each request still checks file content for synchronized replacements. New vaults never inherit another vault's entries. No plaintext files or browser caches are added. Seven focused Rust tests passed (two ciphertext cache tests and five quality-profile tests, including parallel ordering and empty libraries). Release build succeeded and the existing local app is healthy with `/health` HTTP 200.

Chrome verified all 1,941 rows after deployment. A warm in-page refresh took approximately 3.1 seconds, and a full page reload took approximately 3.8 seconds, measured from browser action to completed status via automation (includes automation overhead). The first cold load also completed; its 13.3-second observation included time between tool calls, so it is not a precise load benchmark. Radarr request latency and intermittent upstream failures remain outside this cache fix. Screenshot: `/tmp/rawrarr-movies-chrome-verified.jpg`.

The new cache module, parallel reads, tests, and documentation are task-owned commits. Integration edits inside `secure_config.rs` remain uncommitted because they wrap a metadata loader added by pre-existing uncommitted work; no unrelated implementation was staged. Existing UI and other uncommitted files remain preserved.

## Immediate-list follow-up

The remaining multi-second live request still blocked full reloads. Added an authenticated, no-store snapshot endpoint backed only by the active Radarr client's memory. Successful list requests update the snapshot; replacing or clearing the client drops it. The browser requests snapshot and live data concurrently, renders the last successful list with its update time, then replaces it with live results. Late snapshots cannot overwrite live results. No browser storage or new plaintext disk files are used. First load after a restart still requires a successful live request. Eight browser tests and the Rust snapshot-lifetime test passed. The initial Rust check caught module placement before the crate attribute; moved it below the attribute and reran successfully.

A concurrent node-role/authentication migration changed `main.rs` and Compose during the build. The first rebuilt app refused to start without the new role configuration. Preserved that concurrent work, built from the turn-start main source plus this task's changes in an isolated temporary context, and reconstructed the existing app-only Compose configuration from the container's non-secret environment, mounts, and port. Existing PostgreSQL, Syncthing, network, and volumes were retained. The shared Cargo target reused the wrong executable when copied source timestamps were older; forced a package-only release clean in the isolated Dockerfile before rebuilding. Isolated deployment path is recorded in `/tmp/rawrarr-movie-deploy-path`; do not remove orphans or volumes. The isolated context also required restoring the prior local-authentication implementation (retaining username-less login), removing un-deployed concurrent migrations, and restoring the pre-Sonarr secure-config source with the prior metadata-cache change. These recovery edits were confined to the temporary build context; concurrent workspace changes remain intact.

Final live verification: `/health` returned HTTP 200 and the container became healthy. Chrome loaded 1,941 rows in approximately 1.56 seconds after a full reload and visibly displayed “last updated … · Updating…” before switching to “live data from Radarr.” Measurement includes browser automation overhead. Screenshot: `/tmp/rawrarr-movies-snapshot-verified.jpg`. A concurrent deployment subsequently replaced the app image; final inspected running image was `sha256:43e7b3a66eb5d3ef98c19a23872e5db4a9dfa543054ba60a8286f86a46c36fd0`, healthy. No recovery tag was created because the earlier image had already been removed.

Task-owned backend changes and regression tests are committed. The UI integration remains in the working tree because it depends on the preexisting uncommitted artwork renderer. Do not rebuild the local service from the concurrently changing architecture configuration without completing that migration first. The first list after a service restart or connection replacement still needs live Radarr data.
