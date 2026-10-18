# Browser-reachable Radarr movie links

## Objective and initial state
Make movie posters and titles open their Radarr detail page only when the local browser can contact the configured URL. Existing UI, server, documentation, and test changes were already uncommitted and must be preserved. Port 7297 belongs to `rawrarr-local-rawrarr-1`.

Related: [movie artwork](2026-10-18-radarr-movie-artwork.md), [artwork source](2026-10-18-show-movie-artwork-source.md).

## Actions
- Expose Radarr's title slug with each movie. Use the configured HTTPS base URL, preserving reverse-proxy paths and encoding the slug as one path segment.
- Probe the configured URL from the browser with a three-second timeout, no credentials, no referrer, and no API key. A no-CORS response proves contact; browser/network errors leave the content unlinked. This check does not prove authentication or permission to view a movie.
- Wrap the poster and title in one keyboard-accessible new-tab link only after contact succeeds; refresh checks again. Missing slugs remain unlinked.
- Route confirmed against [Radarr AppRoutes](https://github.com/Radarr/Radarr/blob/develop/frontend/src/App/AppRoutes.tsx).

## Verification and final state
- All five new movie-link tests passed, including actual poster/title popup navigation, browser network failure, a real three-second timeout, missing slug, base-path retention, and refresh rechecking.
- Isolated suite: Node 14/15 passed; separate setup browser test 1/1 passed; Rust 12/12 passed. The unrelated existing brand-case test fails on capitalized Rawrarr text in the pre-existing health page. Python integration tests were skipped by the runner.
- Rebuilt using `RAWARR_POSTGRES_PASSWORD_FILE="$HOME/Library/Application Support/rawrarr/orbstack-postgres-password" docker compose -p rawrarr-local up -d --build rawrarr`; no secret content was read. Container reports healthy and `curl -fsS -o /dev/null -w '%{http_code}' http://127.0.0.1:7297/health` returns 200.
- Live authenticated navigation to the user's Radarr instance was not tested; browser behavior was verified using isolated Chromium fixtures.
- Implementation remains in mixed pre-existing working-tree files (`rust-arr/src/main.rs` and `rust-arr/src/ui.html`): the poster-rendering block itself is uncommitted prior work, so staging it would capture another task. Only the new test, runbook, and task-owned documentation/index additions are committed. Repository is on main with no remote/upstream, so no push is possible. Existing unrelated changes remain untouched.

## Recovery
Revert only this task's changes and rebuild the existing Compose project; preserve volumes and configuration.
