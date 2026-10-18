# Disable UI pages on metadata and storage nodes

## Objective

Keep Rawrarr dashboard and setup page code available for future coordinator use while preventing metadata and storage roles from serving those pages.

## Initial state

The shared server router registered `/`, `/auth`, and `/setup/age` for all roles. The running metadata node therefore had the handlers, while the storage role used the same router code. Existing worktree changes were preserved.

## Actions

- Make UI page routes coordinator-only and return 404 for those paths on metadata and storage roles.
- Rebuild and update the running metadata container only.
- Verify its service health and that UI paths return 404 while health and required API behavior remain available.

## Verification

- The rebuilt metadata container reported `running` and `healthy`.
- On its listener, `/`, `/auth`, and `/setup/age` returned HTTP 404; `/health` returned HTTP 200.
- The coordinator tailnet health route remained available after the metadata-only deployment.
- No storage container was running in the local Compose project. The storage role uses the same subscriber route configuration and will receive these disabled routes when deployed from this source.

## Final state

The dashboard, sign-in, and age setup handlers and templates remain in the source, but only the coordinator role registers them. Metadata and storage roles return 404 for the UI page routes. The metadata deployment lock was removed after verification.
