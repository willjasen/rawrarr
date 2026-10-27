# Fix the config peer status route build

Tracked by [issue #217](https://github.com/willjasen/rawrarr/issues/217).

## Scope

The published `main` commit `1eb5b001fa6f7d719e662738f6e5b2cf5541b949` failed a native AMD64 release image build. The `config_peer_status` local in `main` shadowed the handler function of the same name, so the route passed a `PeerSyncStatus` value to Axum's `get` function.

## Changes

- Rename the handler to `config_peer_status_handler` and update its route registration.
- Update the coordinator workspace regression assertion to cover the route and handler name.

## Verification

- `node --test scripts/tests/test_rawrarr_coordinator_workspace.mjs` — passed (30 tests).
- `git diff --check` — passed.
- Built the exact GitHub branch archive `d656a33` on Brandon’s MacBook Pro with `docker build --platform linux/amd64`; build completed and produced the validation image `rawrarr:validation-d656a33-amd64`. The image was not deployed.
