# Reserve localhost port 7297 for rawrarr-local

## Objective

Use localhost port 7297 for the persistent `rawrarr-local` web service and treat that port as reserved for Rawrarr in the local environment.

## Initial state

The app container was healthy on `127.0.0.1:8084`; PostgreSQL was healthy in the same Compose project. Port 7297 had no listening process, and the only Rawrarr web container was the local app. The unrelated `rawrarr-testing` container was left untouched.

## Changes

- Documented the reserved local port and conflict-resolution rule in `AGENTS.md`.
- Set the Compose host-port default and corresponding default WebAuthn origin to port 7297.
- Updated the user guide to direct local users to `http://localhost:7297/` and resolve conflicts instead of choosing another port.
- Recreated only the `rawrarr` service under the existing `rawrarr-local` Compose project, keeping the existing password-file configuration and shared-data path. PostgreSQL and named volumes were preserved.

## Verification

- Confirmed the app container publishes `127.0.0.1:7297->8080/tcp` and reports healthy.
- Confirmed `GET http://127.0.0.1:7297/health` returns HTTP 200 `ok`; the old port 8084 no longer serves Rawrarr.
- Confirmed the Compose default WebAuthn origin follows the default host port 7297.
- Confirmed PostgreSQL remains healthy and the existing testing container remains running.

## Recovery

The previous published port was 8084. If rollback is needed, recreate only the app service with `RAWARR_HOST_PORT=8084`, preserving its existing secret-file path, shared-data path, and named volumes. Do not remove volumes.
