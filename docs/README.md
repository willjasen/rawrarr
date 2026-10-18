# Rawrarr documentation

Start here for the current project state and everyday use:

- [Project roadmap](../ROADMAP.md) — completed work, outstanding work, and recommended next milestones.
- [Technical boundaries](technical-status.md) — data placement, privacy, and security constraints.
- [User guide](user-guide.md) — persistent local Compose usage, passkey and age setup, HTTPS Radarr connections, artwork, health checks, and local scanning.

## Product design

- [Product and interface design](DESIGN.md) — user-facing principles, flows, privacy presentation, and interaction expectations.

## Architecture and plans

- [Product direction](product-direction.md) — long-term product scope, privacy goals, and compatibility.
- [Node types](node-types.md) — operator service node, subscriber storage node, subscriber shard node, and SwarmSail enrollment.
- [Device signing design](device-signing-design.md) — proposed signed device enrollment and configuration integrity.
- [Data domains](data-domains.md) — placement and synchronization boundaries for `/media`, `/metadata`, `/shared`, `/data`, and backups.
- [Local backups](local-backups.md) — planned manual, age-encrypted backup of `/shared` to one node-local destination.
- [Device architecture](device-architecture.md) — IDs, local/shared records, storage, and remote access boundaries.
- [Cloud access](cloud-access-architecture.md) — passkey account service, blind relay, and subscriber-data boundary.
- [Subscription model](subscription-model.md) — proposed trial, hosted checkout, and signed entitlement model.
- [Tailscale integration](tailscale-integration-plan.md) — optional overlay transport plan; SwarmSail will manage one tailnet per subscriber account.

## Tests and operational notes

- [Testing guide](../TESTING.md) — dependency-only test base, source snapshots, separate Node/Playwright/Rust suites, and integration-test limits.

- [Script and integration test index](../scripts/tests/README.md) — isolated test commands and infrastructure requirements.
- [Runbooks](../runbooks/README.md) — dated implementation and operations records.

- [Mac mini storage-node plan](mac-mini-storage-node-plan.md) — OrbStack deployment, attached-drive access, same-instance bootstrap, and acceptance checks.
