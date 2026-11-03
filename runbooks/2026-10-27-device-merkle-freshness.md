# Per-device Merkle metadata freshness

Issue: [#100](https://github.com/willjasen/rawrarr/issues/100)

## Scope

Expose each metadata node's latest verified config-revision Merkle agreement on the coordinator Devices page. The node computes status after its periodic signed peer-manifest reconciliation and reports `current` only when the final local Merkle root matches every configured peer whose manifest passed validation and all configured peers were reachable. Partial agreement, no agreement, no configured peers, and not-yet-checked status remain distinguishable.

## Implementation

- Added an in-memory node sync status snapshot updated by `PeerRpc::reconcile_once`.
- Exposed it through the subscriber's authenticated `/api/config-peer/status` route and allowed that read-only route through the coordinator's existing owner-authenticated node relay.
- Added a badge to metadata device cards with a distinct icon and text for current, incomplete, unreachable, and checking states.
- Added regression assertions for the per-node route and UI states, plus three-node partial-versus-full peer agreement.

## Verification

- `node --test scripts/tests/test_rawrarr_coordinator_workspace.mjs` — passed (29 tests).
- `git diff --check` — passed.
- Rust `config_repair` suite — not run: this host has no `cargo` executable, and the isolated OrbStack runner cannot reach the Docker socket in the restricted shell.
