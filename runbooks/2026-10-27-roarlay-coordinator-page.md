# Roarlay coordinator page

Issue: https://github.com/willjasen/rawrarr/issues/257

## Scope

Add a dedicated Roarlay page to the authenticated coordinator workspace. The page identifies the current instance as the coordinator and checks the same-origin `/health` endpoint. Subscriber enrollment, topology, and management remain in Fossils.

## Adoption

This is an internal coordinator UI change. Publish the change through the normal GitHub `main` workflow, then deploy the exact published revision to the internal `roarlay` service on do3. Mac fossil nodes do not need an update.

## Migration and compatibility

No migration is needed. The change adds no persistent data, schema, encrypted configuration, or node/coordinator protocol state. Existing deployments adopt the page when the updated coordinator UI is deployed. The health panel uses the existing `/health` route and is safe alongside older subscriber nodes.

## Verification

Run the focused coordinator workspace regression and the browser topology/health regression. Verify the page at `#roarlay`, responding and unavailable health states, and that the Fossils view remains separate.

## Rollback

Redeploy the prior published coordinator revision. No data rollback is needed.
