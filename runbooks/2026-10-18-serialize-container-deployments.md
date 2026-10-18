# Serialize Rawrarr container deployments

## Objective

Prevent concurrent tasks from rebuilding or deploying the same Rawrarr container while allowing independent containers to be deployed concurrently.

## Changes

- Documented one hidden, empty-capable marker per Compose container under `~/Library/Application Support/Rawrarr/`, outside synchronized `shared-data`. A marker name includes the target container, such as `.container-rebuild-coordinator.lock`, `.container-rebuild-metadata.lock`, or `.container-rebuild-storage.lock`.
- A deployment task checks and atomically claims only the target container's marker, waits without changing that container if another task owns it, keeps the marker through verification, and removes it when finished. Independent container deployments may proceed concurrently unless one deployment would restart or disrupt the other's locked container.
- Runtime markers are local-only and must never be committed to Git. Added `.container-rebuild-*.lock` to the repository `.gitignore` as defense in depth if a marker is accidentally created in the checkout.

## Verification

Reviewed the instruction placement and checked the patch for whitespace errors. No containers or application data were changed.

## Recovery and remaining work

If a task exits unexpectedly, first verify that no build, restart, or deployment operation remains active before removing a stale marker. Then allow waiting tasks to continue.
