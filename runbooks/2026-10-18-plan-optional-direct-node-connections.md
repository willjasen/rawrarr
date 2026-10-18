# Plan optional direct node connections

## Objective

Prepare an implementation plan for browser-to-node direct access when reachable, retaining the coordinator route as fallback across networks.

## Initial state

- The worktree contained extensive pre-existing modifications and untracked files. No existing changes were staged by this task.
- Existing architecture documents describe the coordinator workspace proxy, isolated node routes, a future blind relay, and direct-peer preference as separate states.
- No runtime or deployment state was changed.

## Actions

- Reviewed the current device, cloud access, coordinator linking, and blind-content architecture documents and roadmap.
- Added [the optional direct node connection plan](../docs/direct-node-connection-plan.md), covering current state, proposed flow, prerequisites, implementation order, security constraints, and acceptance criteria.
- Preserved the distinction between the current trusted self-hosted coordinator proxy and the future blind public relay.
- Recorded the user's decisions: Tailscale-only initially (defer same-LAN), automatic route selection with visible status, and a design that covers both current self-hosted use and the future hosted portal in separate milestones.

## Verification

- Documentation links point to existing architecture documents.
- The plan explicitly retains coordinator fallback and does not claim direct connectivity is implemented.
- No tests were run because this task changes documentation only.

## Remaining technical decisions

- Whether to offer a manual coordinator-route override.
- Tailscale browser membership, node DNS names, certificate issuance/renewal, and route probe/fallback timing.
- Hosted blind-relay TLS, ticket, node-origin identity, and abuse-control design.
