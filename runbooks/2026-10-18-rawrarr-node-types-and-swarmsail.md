# Define Rawrarr node types and SwarmSail role

## Objective and initial state

Clarify the deployment roles after the user specified that the operator's main node should eventually run SwarmSail and coordinate subscriber Headscale tailnets. Existing plans covered a blind relay/subscription service and subscriber web/storage nodes but did not establish one shared role vocabulary. Reviewed [device architecture](../docs/device-architecture.md), [cloud access architecture](../docs/cloud-access-architecture.md), [subscription model](../docs/subscription-model.md), [Tailscale integration plan](../docs/tailscale-integration-plan.md), and the existing storage-less web-node runbook.

## Architecture recorded

- Named three Docker deployment roles: operator `service`, subscriber `storage`, and subscriber media-less `shard`.
- Defined storage and shard nodes as full Rawrarr web/API nodes, separated by the presence of a local media mount.
- Defined the service node as SwarmSail/Headscale coordination plus hosted account/subscription and future blind-relay services. It remains outside subscriber shared folders and does not hold private age identities, app-data plaintext, or media.
- Recorded instance-based subscription and per-node, transient Headscale enrollment direction; Tailscale carries app/control traffic only, never media.
- Added open decisions for isolated-tailnet granularity, service deployment boundary, device limits/revocation, policy rules, and safe one-time enrollment delivery.
- Added the canonical [node types architecture](../docs/node-types.md) and linked it from the existing architecture docs and runbook index.

## Verification and remaining work

Documentation only. No Docker modes, SwarmSail service, Headscale tailnet, subscriber node, or live container changed. The proposed environment variable `RAWRARR_MODE` is a design placeholder, not an implemented or frozen interface. `git diff --check` and relative Markdown link checks passed. SwarmSail automation API behavior and tenant isolation remain to be reviewed before implementation.
