# Rebuild rawrarr-local from the current checkout

## Objective

Update the persistent `rawrarr-local` app container from the current project source so Chrome serves the latest setup flow.

## Initial state

Chrome at `http://localhost:8084/setup/age` showed the prior challenge asking for unrelated positions. The `rawrarr-local-rawrarr-1` container was healthy but used an older `rawrarr:local` image. Compose initially could not interpolate the PostgreSQL secret-file variable in the shell environment; the container remained unchanged.

## Actions

- Inspected the current container's non-secret runtime configuration and mount paths. Kept its existing environment, service port, data volumes, shared-data bind, and secret mount configuration in process memory; did not read secret contents or print secret values.
- Rebuilt and recreated only the Compose `rawrarr` service with `--no-deps`, leaving PostgreSQL running and preserving its volumes.
- Reloaded Chrome's open setup tab after the service started.

## Verification

- The app container reported `running` and `healthy`.
- `http://localhost:8084/health` returned HTTP 200.
- The running binary contains the new consecutive-character prompt and not the old random-position prompt.
- After refreshing Chrome, its accessibility tree showed the current setup page (`Create ID`, current private-key instructions) instead of the old copy.

## Recovery and remaining work

The previous image remains available in the local Docker image store for manual rollback. The browser refresh cleared the page's in-memory identity; the user must paste their saved identity again to resume setup. Future source changes require rebuilding the `rawrarr` service from this checkout.
