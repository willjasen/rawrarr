# Check coordinator UI releases every five minutes

## Objective

Return the coordinator UI updater to a five-minute polling interval. The updater remains restricted to coordinator-role containers. GitHub issue [#169](https://github.com/willjasen/rawrarr/issues/169).

## Changes

- Set the updater default and minimum poll interval to 300 seconds.
- Added focused regression coverage for the default, floor, and a longer requested interval.
- Updated the script test index.

## Verification

- `python3 scripts/tests/test_ui_bundle.py -v` passed all 6 tests, including the five-minute default/floor and longer custom interval.
- `git diff --check` passed.

## Deployment

PR [#171](https://github.com/willjasen/rawrarr/pull/171) merged at `ef2fa2752496ff09ce5f50ed0444d88e47b3d535`. Before deployment, authenticated `gh` confirmed published `main` at `6988fa1fb60b38181bfd72d73af81eeec1ddbbad`; local `main` was synchronized to that commit. The only commit after the PR merge was documentation-only.

The last recorded internal source baseline was `99e2e82d2e1953950a7820800018a37500618919`. The deployment classifier selected both coordinator and node because published `main` also contained shared runtime updates since that baseline. The Mac mini's cached GitHub checkout could not authenticate, so the tracked source archive was created from the verified published commit, transferred to both Macs, and checked against SHA-256 `374084718c2e9805e3f933238437d57505671f1fbd2dc794368736096148a185` before building.

- Built the coordinator source natively for ARM64. Its image ID (`sha256:887b85385ca6df5b5ad5dba7f7380355349e6959ce59b9bf27deda0d222775e0`) matched the already-running roarlay image, so roarlay did not need recreation. The live updater reports a 300-second default and minimum. The prior image remains tagged `rawrarr:rollback-roarlay-6988fa1`.
- Built the subscriber source natively for AMD64 as image `sha256:5dc35b86f2b3b6e04bc44816ef6dcb30a7ab9c1b2859cacd291378d9eae7211c`, preserving the previous image as `rawrarr:rollback-fossil-6988fa1`. Recreated only `fossil` in `rawrarr-node-internal` with `--no-deps --force-recreate`; Valkey and Tailscale were not recreated.
- Both per-container deployment markers were atomically claimed, held through the deployment and verification, then removed and confirmed absent.

## Deployment verification

- Roarlay is running and healthy with zero restarts. Its updater reports default/minimum interval `300` seconds. `https://rawrarr.risk-mermaid.ts.net/health` returned HTTP 200 and identified Roarlay.
- Tailscale Serve read-back confirms `rawrarr.risk-mermaid.ts.net:443` `/` proxies to `http://127.0.0.1:8080`. PostgreSQL and the Tailscale sidecar were not recreated.
- Fossil is running and healthy with zero restarts on image `sha256:5dc35b86f2b3b6e04bc44816ef6dcb30a7ab9c1b2859cacd291378d9eae7211c`; its target architecture marker is `amd64`. Its local health endpoint and HTTPS health route returned HTTP 200. The Tailscale sidecar container ID remained unchanged.
