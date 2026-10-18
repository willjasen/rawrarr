# Central coordinator, storage, and metadata containers

## Objective and initial state

Implement the user's revised three-role model and prepare Mac mini installation. Coordinator is central passkey/authentication and node coordination only; storage and metadata both serve the user app, Radarr, and shared app data; only storage reads media. This supersedes the role assumptions in [the prior linking plan](2026-10-18-coordinator-storage-device-linking.md).

The existing `rawrarr-local` app on loopback port 7297 combines authentication and subscriber data, with a separate PostgreSQL volume and Syncthing companion. Extensive pre-existing tracked/untracked changes on `main` must remain intact. No remote is configured. Initial port ownership: `rawrarr-local-rawrarr-1`. No Mac mini deployment is implied by local validation.

## Implementation

In progress. Split startup/routes and authentication, add role-specific Compose and protected manual node provisioning, test isolated deployments before migrating the persistent local app. Keep existing volumes, browser windows, passkeys, shared folder, and remembered age identity. No private identities or Radarr settings may enter coordinator paths.

## Verification and recovery

Pending. Before local migration, retain an image rollback tag and protected PostgreSQL backup. No volume deletion. Device-code enrollment, multi-account tenancy, and remote network publication are separate work.


## Docker Desktop role names

Renamed the local Compose services to `coordinator`, and `metadata` (PostgreSQL remains the `postgres` supporting service). The standalone Compose services are named `postgres`/`coordinator`, `metadata`, and `storage`; the local media overlay keeps the `metadata` service name while selecting the storage role. Explicit named volumes retain their original names under the existing `rawrarr-local` Compose project.

Recreated the local coordinator and metadata services under these names. PostgreSQL retains its existing service name, data volume, and protected credentials; metadata retains the existing app data volume and shared bind. The previous `rawrarr` app container was stopped and removed without volume deletion, then the renamed `metadata` service was started on loopback port 7297.


Compose initially tried to run two PostgreSQL processes against the same named data volume after changing the PostgreSQL service key. Both newly created coordinator services were stopped; the original database and metadata containers remained available. The PostgreSQL service key was restored to `postgres` to avoid concurrent writers while keeping the subscriber services role-named.

## Verification

`docker compose -p rawrarr-local config --quiet` passed with the protected password-file path set. Docker Desktop's Compose services are now `coordinator` and `metadata`; both report healthy, and both loopback health routes return HTTP 200 (`7298` and `7297`, respectively). The existing `rawrarr-data` and `rawrarr-postgres-data` named volumes remain present; PostgreSQL and Syncthing remain healthy. The old `rawrarr` app container was removed after stopping it; its named volume was retained. `git diff --check` passed. The Mac mini setup is documented but has not been run.

No task-only commit was made because these Compose, docs, and source files already contain unrelated user changes and the role implementation predates this naming task. Do not stage those mixed files wholesale.
