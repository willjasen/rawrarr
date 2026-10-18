# Preserve the current page on refresh

## Objective and initial state

Browser refresh reset Movies to Overview because the selected view was only held in the DOM. The checkout contains extensive unrelated changes, which are preserved. The existing `rawrarr-local-rawrarr-1` container owns `127.0.0.1:7297`.

Related deployment context: [Local port](2026-10-18-rawrarr-local-port-7297.md).

## Actions

- Persist Overview, Movies, and Configuration in URL fragments; restore on startup and browser history navigation.
- Unknown or empty fragments display Overview. Settings opens Overview before focusing the connection form.
- Add a browser regression test for reload, in-page refresh, history, direct links, fallback, and Settings.

## Verification and final state

- Chromium regression test passed (1 test), running the actual UI source with stubbed API responses in the existing testing image. Covered browser reload, Refresh movies, history, direct links, unknown fragments, and Settings focus.
- Rebuilt only the app with `RAWARR_POSTGRES_PASSWORD_FILE="$HOME/Library/Application Support/rawrarr/orbstack-postgres-password" docker compose -p rawrarr-local up -d --build --no-deps rawrarr`; existing volumes and PostgreSQL preserved.
- `/health` returned HTTP 200 after restart; Docker reports the app running and healthy. No authenticated live-browser or live Radarr integration test was performed.
- `git diff --check` passed. Task changes committed locally on `main`; no remote/upstream is configured. Pre-existing modifications and untracked files remain outside this commit.

## Recovery

Revert only this task's navigation changes and rebuild the existing Compose service; retain all data and volumes.
