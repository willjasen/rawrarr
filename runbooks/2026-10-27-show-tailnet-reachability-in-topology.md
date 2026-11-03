# Show tailnet reachability in topology

## Objective
Color device links green when one node successfully reaches its configured Rawrarr peer over the tailnet, independently of Syncthing's folder connection state. Tracked by GitHub issue #259.

## Implementation
The metadata peer reconciliation loop already performs a signed challenge and manifest request to configured peers. Its status response now includes the peer IDs for which the signed manifest endpoint responded successfully. The coordinator UI uses this signal in either direction to color the pair green and retains Syncthing state as a fallback when no tailnet peer status is available. Existing nodes remain compatible: older status responses omit `reachablePeers` and continue to use the prior Syncthing display behavior. No persisted data or protocol migration is required; the added JSON property is optional and backward compatible.

Reachability is reported only for peers configured in `RAWARR_CONFIG_PEERS`; an absent or stale report remains unconfirmed. This measures an authenticated Rawrarr request over the configured peer URL, not merely Tailscale membership or ICMP reachability.

## Verification
`node --test scripts/tests/test_rawrarr_coordinator_workspace.mjs` passed (33/33). The isolated OrbStack regression runner passed all Node tests (137/137), Playwright browser tests (all files, including `test_rawrarr_topology_visibility_browser.mjs`), and Rust unit tests (113/113). Its first Rust run exposed that the existing signed-peer fixture does not reach the manifest path in the test container; the regression now tests `reachablePeers` serialization directly. Python integration tests were skipped by the runner because they require separate fixtures. `git diff --check` passed.

## Adoption and rollback
Deploy the coordinator UI and updated metadata node application from a published `main` revision using the established internal deployment workflow. UI-only coordinator deployments remain isolated to internal roarlay. Older nodes continue to omit the optional field. Rollback is an ordinary application image rollback; no data migration is needed.
