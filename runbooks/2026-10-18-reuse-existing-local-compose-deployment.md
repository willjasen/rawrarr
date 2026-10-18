# Reuse the existing local Compose deployment

## Objective and initial state

Validate that this standalone checkout can manage the existing `rawrarr-local` OrbStack project without moving its data, then restore the web service if its existing container is stopped. The password file and shared-data directory remain outside the repository.

## Inspection

- The existing app and PostgreSQL containers use project `rawrarr`; Compose labels identify `rawrarr-local` and the old `arr-services/docker-compose.yml` as their project.
- The app's `/data` and PostgreSQL data use named volumes `rawrarr-local_rawrarr-data` and `rawrarr-local_rawrarr-postgres-data`.
- `/shared` binds to `/Users/willjasen/.binds/sync/Codebox/arr-services/shared-data`; the password file binds from `/Users/willjasen/Library/Application Support/Rawrarr/orbstack-postgres-password`.
- The shared folder contains `age-recipient` by filename only. No identity contents were read.
- Compose validation from this checkout passed with project `rawrarr-local`, host port `8084`, and the existing password and shared-data paths explicitly set.
- At inspection, PostgreSQL was healthy and the app container was exited with code 137; the HTTP health endpoint could not connect. The exit code alone does not identify why it stopped.

## Recovery action

After confirming the project name, port, secret-file path, shared-data bind, and named-volume identities, start the web service using this checkout's Compose file and the explicit existing paths:

```sh
RAWARR_HOST_PORT=8084 \
RAWARR_POSTGRES_PASSWORD_FILE='/Users/willjasen/Library/Application Support/Rawrarr/orbstack-postgres-password' \
RAWARR_SHARED_DATA_PATH='/Users/willjasen/.binds/sync/Codebox/arr-services/shared-data' \
docker compose -p rawrarr-local -f docker-compose.yml up -d rawrarr
```

Do not use `down -v`; it would remove the named data volumes. Keep runtime data and credentials outside Git.

## Verification and remaining work

The app container had exited with code 137 while PostgreSQL remained healthy. After confirming the configured project, host port, secret path, shared-data bind, and named-volume identities, `docker compose ... up -d rawrarr` from this checkout recreated and started only the app service. The app is running and `http://localhost:8084/health` returned `ok`; PostgreSQL remains healthy. Inspection confirmed the app still mounts the original shared-data directory, the existing password file, and the existing `rawrarr-local_rawrarr-data` volume; PostgreSQL still uses `rawrarr-local_rawrarr-postgres-data`. `docker compose config --quiet` and `git diff --check` passed. No data migration or PostgreSQL-volume change was performed.
