# Improve the Rawrarr health page

## Objective

Replace the plain `ok` response at `/health` with a clear, styled status page.

## Initial state

`GET /health` returned the plain text `ok`. The persistent `rawrarr-local` application container owned `127.0.0.1:7297` and was healthy; PostgreSQL was also healthy. The checkout contained unrelated user changes, which were preserved.

## Changes

- Render a responsive dark status page at `/health` with operational state, web-service check, last-checked time, auto-refresh, and a link back to Rawrarr.
- State explicitly that this endpoint checks the web process and does not represent the health of integrations or other nodes.

## Verification

- Rebuilt and recreated only the existing `rawrarr-local` app service using its existing project, password-file path, shared-data bind, and named volumes; PostgreSQL was not recreated.
- `GET http://127.0.0.1:7297/health` returned HTTP 200 and `text/html`; the response contains the status heading, service checks, auto-refresh note, and dashboard link.
- Docker reports the app container healthy. `git diff --check` passed for the task-owned files.

## Final state

The `/health` endpoint now presents a responsive service status page. No authenticated browser rendering or component integration checks were performed; the page explains this limit.
