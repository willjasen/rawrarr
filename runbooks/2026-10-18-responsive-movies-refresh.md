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
