# Define SwarmSail tailnet account boundary

## Objective and initial state

Capture the user's decisions about the planned SwarmSail integration after reviewing [node types](../docs/node-types.md) and [cloud access architecture](../docs/cloud-access-architecture.md).

## Decisions recorded

- Create one isolated SwarmSail/Headscale tailnet per subscriber account.
- Use Tailscale as an overlay network only for now. Tailnet membership does not imply permission to read Rawrarr shared data; Rawrarr authorization and age recipients remain the data access controls.
- Defer whether an invited collaborator joins the owner's tailnet or remains in a separate tailnet behind a limited bridge.
- Retain instance-based Rawrarr subscription as the current proposed billing unit; account-to-multiple-instance behavior needs product decisions if supported later.

## Verification and remaining work

Updated the architecture docs and runbook index. This is planning only: no SwarmSail tenant, Headscale tailnet, ACL, or application code changed. Relative Markdown link checks and `git diff --check` passed.
