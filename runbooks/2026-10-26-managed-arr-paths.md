# Managed Radarr and Sonarr UI paths

## Objective

Track the design and implementation of Rawrarr paths for Radarr and Sonarr containers managed inside a subscriber's Docker environment. External Arr endpoints configured for metadata access are explicitly out of scope. GitHub issue: [#116](https://github.com/willjasen/rawrarr/issues/116). Related provisioning work: [#96](https://github.com/willjasen/rawrarr/issues/96).

## Initial state

- Started from fetched `origin/main` at `bdcadb09a999e0ecd45b6f28c67adfc75c22610e` in a dedicated worktree and branch `codex/internal-arr-path`.
- The coordinator has authenticated node-scoped proxy paths, while subscriber Compose generation does not currently provision Arr apps. The OrbStack test stack has private Radarr/Sonarr services.
- Existing app settings connect to Arr endpoints for metadata; they must not become proxy targets or determine managed-app availability.
- Existing route and connection context: [documented Tailscale proxy routes](2026-10-18-document-tailscale-proxy-routes.md), [Radarr connection save](2026-10-18-radarr-connection-save.md), [subscriber Compose plan](../docs/one-line-node-deployment-plan.md), and [managed provisioning issue #96](https://github.com/willjasen/rawrarr/issues/96).

## Actions

- Created issue [#116](https://github.com/willjasen/rawrarr/issues/116) with acceptance criteria for internal-only upstreams, private networking, authentication, UI links, and coverage.
- Added [the managed Arr paths design](../docs/managed-arr-paths-plan.md). The implementation will coordinate the private service and URL-base contract with issue #96 before proxying broad Arr UI traffic.

## Verification

- Reviewed the current coordinator node-proxy allowlist, subscriber API router, subscriber Compose generator documentation, the isolated OrbStack Arr services, and issues #95/#96.
- No application code or live services changed. Tests and live-route checks have not been run.

## Current status and remaining work

Planning/design milestone is recorded; implementation remains open. Next, align the managed service Compose contract with issue #96, then implement and verify the scoped proxy, UI links/status, and security/availability cases. No production deployment is authorized by this task.
