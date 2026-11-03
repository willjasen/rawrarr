# Fossils loading independently of subscriber diagnostics

Issue: https://github.com/willjasen/rawrarr/issues/365

## Diagnosis

The coordinator workspace waited for every subscriber's Syncthing health,
config-peer status, direct-route probe, and encrypted device-name lookup before
rendering its node cards. The relay health and encrypted-name requests had no
browser deadline. A stalled subscriber could therefore leave Fossils blank and
prevent initial startup from reaching the polling registration.

The internal coordinator HTTPS health route responded during investigation.
Chrome exposed the requested URL but no readable page accessibility tree, so
the current live blank state could not be inspected with the UI tool.

## Change

Render the authenticated coordinator node list before optional diagnostics.
Run Syncthing, Merkle, and direct-route checks concurrently. Bound the node list,
relay diagnostics, pending list, and encrypted-name network requests to five
seconds; direct-route probes retain their existing 2.5-second limit. Keep one
list refresh in flight and ignore late name results from an older node snapshot,
a local name edit, or a cleared/replaced age identity. Names load independently of diagnostic
completion, retaining existing labels on failure and reporting that failure in
the browser console.

Cards show checking state before diagnostics arrive. Default node selection
still waits for direct-route measurements so the fastest usable direct route
remains preferred; explicit user selection is available immediately.

## Adoption and rollback

This is a coordinator browser-asset change with no persistent-data, schema,
encrypted-file-format, or node protocol change. No migration is required.
Existing subscribers remain compatible. Publish and merge the focused branch
before deployment; deploy only the exact GitHub main revision to the internal
do3 roarlay service, preserving PostgreSQL, the dedicated Tailscale sidecar,
public services, and Mac subscribers. Reload an existing workspace to adopt the
new assets. Rollback uses the prior coordinator image/assets; no data rollback
is needed.

## Verification

Rebuilt the committed browser export bundle with `npm run build --prefix
rust-arr/web-export` after restoring its locked dependencies with `npm ci`.

Ran `python3 scripts/tests/run_rawrarr_tests_in_docker.py` against the final
working tree using the existing reusable base. Node.js checks passed and all
116 Rust checks passed. All seven topology-visibility Chromium checks passed,
including holding Syncthing, Merkle, direct health, and encrypted names pending,
checking cards and Edit within 1.5 seconds at 1280x800 and 375x812, observing
timeout states, and verifying a later poll recovers names and Merkle status
without losing selection.

The full browser suite still reports three existing failures: direct
Radarr/Sonarr settings text, outdated Docker-runner fixture assertions, and
signing-topology initial viewport placement. Ran the same Docker runner against
a clean archive of unchanged `e14b8ba` and reproduced all three failures.
They are outside this fix and were not modified. Python integration and
multi-node environment tests are not included in this runner and were not run.

Live deployment and authenticated Chrome behavior have not been verified.
