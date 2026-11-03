# Fix Merkle peer status counts

Issue: [#231](https://github.com/willjasen/rawrarr/issues/231)

## Scope

Correct the per-device Merkle freshness count when a node's peer map contains its own node ID. The configured-peer denominator must count remote peers only, while retaining unreachable remote peers in the denominator.

## Implementation

- Added a single remote-peer count that excludes the local node ID.
- Used that count both when certificate setup is unavailable and after reconciliation, so matching count and configured count describe the same remote peer set.
- Added regression coverage for a self entry and one remote peer.

## Verification

- `python3 scripts/tests/run_rawrarr_tests_in_docker.py --context orbstack --remove-after` — passed (127 Node tests, Playwright browser suite, and 109 Rust tests).
- `git diff --check` — passed.
