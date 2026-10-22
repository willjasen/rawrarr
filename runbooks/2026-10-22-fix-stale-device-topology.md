# Fix stale device topology labels

## Objective

Keep the coordinator Topology diagram aligned with the current device names and identity labels shown in Devices.

## Initial state

The screenshot showed an enrolled device displayed by raw node ID in Topology while its device card used a friendly display name. The diagram loaded current enrolled nodes but used only the age-encrypted alias or node ID for labels and peer matching. Existing deployment instructions require updating only roarlay on the Mac mini and verifying the HTTPS tailnet route.

## Changes

Topology labels now use the same fallback as device cards: saved encrypted alias, coordinator display name, then node ID. Peer matching also considers the coordinator display name so Syncthing's peer name can match an enrolled device when the encrypted alias is unavailable.

## Verification and deployment

The focused topology source test, JavaScript syntax check, and diff whitespace check passed. The full coordinator workspace test file has unrelated failures against existing concurrent source/test changes. Tailnet `/health` responds. Deployment was not completed: SSH to the Mac mini succeeded, but its shell could not find the Docker CLI used to manage the coordinator. No service state was changed.

## Recovery and remaining work

Revert the two topology fallback expressions in `rust-arr/src/coordinator-workspace.mjs` to restore prior behavior. To show the fix on the live coordinator, rebuild only `roarlay` on the Mac mini under its deployment lock using the established local Docker environment, then verify roarlay health and the HTTPS Serve route. Do not restart PostgreSQL or Tailscale.
