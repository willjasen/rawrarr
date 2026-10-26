# Rawrarr documentation

- [Container deployment](container-deployment.md) — implemented roarlay (coordinator), fossil (metadata), and boneyard (storage) roles and Mac mini setup.
- [Security](SECURITY.md) — trust boundaries, age encryption, browser-to-node encrypted transport through the coordinator, authentication, storage, and current limitations.
- [Production deployment plan](production-deployment-plan.md) — start with coordinator-local PostgreSQL, production launch requirements, and a future managed-database migration path.
- [Admin console](admin-console.md) — private, separate-container dashboard preview, network boundaries, and the authentication gate required before live account data.

Start here for the current project state and everyday use:

- [Project roadmap](../ROADMAP.md) — completed work, outstanding work, and recommended next milestones.
- [Technical boundaries](technical-status.md) — data placement, privacy, and security constraints.
- [User guide](user-guide.md) — persistent local Compose usage, passkey and age setup, Radarr connections, Movies/TV summaries, artwork, health checks, and local scanning.

## Product design

- [Product and interface design](DESIGN.md) — user-facing principles, flows, privacy presentation, and interaction expectations.

## Architecture and plans

- [Product direction](product-direction.md) — long-term product scope, privacy goals, and compatibility.
- [Shared cluster plan](shared-cluster-sharing-plan.md) — isolated/shared modes, owner-controlled membership, MLS evaluation over Syncthing, age-encrypted files, revocation cleanup, and implementation phases.
- [Node types](node-types.md) — roarlay, fossil, and boneyard node roles, plus SwarmSail enrollment.
- [One-line node deployment plan](one-line-node-deployment-plan.md) — planned curl-based installer containing generated Compose, Docker prerequisite setup, safe reruns, and explicit owner approval.
- [Device signing design](device-signing-design.md) — proposed signed device enrollment and configuration integrity.
- [Identity domains](identity-domains.md) — agreed user, device, group, and app key boundaries, with current and planned lifecycle clearly separated.
- [Data domains](data-domains.md) — placement and synchronization boundaries for `/media`, `/metadata`, `/shared`, `/data`, and backups.
- [Radarr recovery](user-guide.md#recover-radarr-on-a-chosen-node) — age-encrypted shared native backups and user-directed restores to a Radarr deployed on the chosen node.
- [Shared config schema v1](shared-config-v1.schema.json) — versioned age-encrypted config document with Radarr and Sonarr entries in `services`.
- [Metadata export plan](metadata-export-plan.md) — browser-side age decryption and ZIP export of indexed movie metadata; includes remaining media classes and scale limits.
- [Artwork cache plan](artwork-cache-plan.md) — node-local Redis-compatible Valkey cache for encrypted artwork, sizing, stale-entry handling, operations, and rollout acceptance criteria.
- [Blind coordinator content flow](blind-coordinator-content.md) — keep metadata/storage content encrypted through the coordinator and decrypt it in the user's browser; documents the bounded implementation and remaining scope.
- [Browser security test plan](security-test-plan.md) — plan Playwright integration evidence for metadata/storage confidentiality through the coordinator, including limits and malicious-coordinator caveats.
- [Public landing page](DESIGN.md#account-and-first-run-flow) — public, pre-sign-in introduction with clear Rawrarr maturity and security-limit disclosures.
- [Public demo plan](public-demo-plan.md) — planned account-free tour using isolated synthetic data, read-only interactions, and explicit product-maturity disclosures.
- [Local backups](local-backups.md) — planned manual, age-encrypted backup of `/shared` to one node-local destination.
- [Device architecture](device-architecture.md) — IDs, local/shared records, storage, and remote access boundaries.
- [Cloud access](cloud-access-architecture.md) — passkey account service, blind relay, and subscriber-data boundary.
- [Optional direct node connections](direct-node-connection-plan.md) — direct Tailscale route and coordinator fallback, with remaining transport and security work.
- [Subscription model](subscription-model.md) — proposed trial, hosted checkout, and signed entitlement model.
- [Donation plan](donation-plan.md) — proposed optional one-time contributions, separate from subscription entitlements.
- [Tailscale integration](tailscale-integration-plan.md) — optional overlay transport plan; SwarmSail will manage one tailnet per subscriber account.
- [Account-bound subscriber tailnet provisioning](account-tailnet-provisioning-plan.md) — planned eligibility, isolation, provisioning, key lifecycle, and revocation requirements.

## Tests and operational notes

- [Testing guide](../TESTING.md) — dependency-only test base, source snapshots, separate Node/Playwright/Rust suites, and integration-test limits.

- [Script and integration test index](../scripts/tests/README.md) — isolated test commands and infrastructure requirements.
- [Runbooks](../runbooks/README.md) — dated implementation and operations records.

- [Mac mini boneyard-node plan](mac-mini-storage-node-plan.md) — OrbStack deployment, attached-drive access, same-instance bootstrap, and acceptance checks.

- [Coordinator device linking](coordinator-linking-plan.md) — Planned Docker-log device code, coordinator sign-in and approval, and authenticated storage enrollment.
