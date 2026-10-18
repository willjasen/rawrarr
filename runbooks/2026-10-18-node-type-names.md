# Name Rawrarr node types

## Objective

Adopt **roarlay** as the user-facing name for the coordinator role and **apex** for storage nodes. Leave metadata nodes unnamed until Brandon chooses a term.

## Context

The node roles already have distinct capabilities documented in [node types](../docs/node-types.md). This task changes vocabulary only; current Compose service names and internal role identifiers remain `coordinator`, `storage`, and `metadata`. The worktree contained extensive pre-existing changes, which were preserved.

## Actions

- Updated the node types overview to distinguish user-facing names from internal identifiers and to mark the metadata name as undecided.
- Added this task record and linked it from the runbook and documentation indexes.

## Verification

- `node --test scripts/tests/test_rawrarr_coordinator_workspace.mjs scripts/tests/test_rawrarr_coordinator_passkey.mjs scripts/tests/test_rawrarr_node_approval_ui.mjs scripts/tests/test_rawrarr_direct_route.mjs`: 16 passed.
- Compose config validation passed for the local roarlay/Tailscale stack and standalone apex service.
- The local `rawrarr-local-roarlay-1` container reports healthy. `https://rawrarr.risk-mermaid.ts.net/health` identifies roarlay, and Serve status maps HTTPS 443 `/` to `http://127.0.0.1:8080`; `/direct-api` remains mapped to `http://127.0.0.1:8082`.
- Confirmed the MacBook Tailscale container retains the `coordinator` network alias. The old stopped `rawrarr-local-coordinator-1` container was removed; persistent Postgres and metadata containers remain running.
- `git diff --check` passed.

## Final state

The app UI and Docker services use roarlay and apex. Metadata remains “metadata” pending a naming decision. Internal `RAWARR_ROLE` values, database values, API paths, and stable node IDs remain compatible. The Tailnet app container was deployed and verified; no volume or browser changes were made.

## Implementation update

Updated user-facing app labels to roarlay/apex and renamed Compose services/container names to `roarlay` and `apex`, including the local Compose service (`roarlay`) and its generated container name (`rawrarr-local-roarlay-1`). Internal role values, database values, API routes, stable IDs, and metadata naming remain unchanged.
