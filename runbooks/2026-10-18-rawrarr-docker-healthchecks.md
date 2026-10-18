# Report Rawrarr container health

## Objective

Make both containers in the local Rawrarr Compose project report Docker health based on service readiness.

## Initial state

PostgreSQL already had a `pg_isready` health check. The Rawrarr application container had no health check, even though the service exposes `/health`.

## Changes

- Install `curl` in the final application image and add an image-level HTTP check against `http://127.0.0.1:8080/health`.
- Add a Node test that guards both the application image check and PostgreSQL Compose check.

## Verification

- `node --test scripts/tests/test_rawrarr_docker_healthchecks.mjs` passed.
- `git diff --check` and Docker Compose config validation passed.
- Docker image rebuild succeeded and Compose recreated the Rawrarr service.
- `docker compose ... ps` reports both `rawrarr` and `postgres` as `healthy`; `http://localhost:8084/health` returned `ok`.

## Final state

The application image has an HTTP health check against its `/health` route, and PostgreSQL retains its `pg_isready` check. Both local containers report healthy.
