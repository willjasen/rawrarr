# Compact completed artwork cache status

## Objective

Reduce the Movies page artwork status after all eligible posters have been cached.

## Initial state

The UI always displayed the detailed saved/total count, including after the backfill completed. The working tree contained extensive unrelated changes; edits were limited to the artwork status UI, its focused regression test, and this runbook/index entry.

## Changes

- Keep the detailed count and progress bar while artwork is actively saving.
- Once inactive and saved count reaches the total, hide the progress bar and show a compact “Artwork cache complete” status. Preserve any unavailable count.
- Keep detailed status when the cache is not complete and work is inactive.

## Verification

- Focused regression test passed: `node --test scripts/tests/test_rawrarr_artwork_backfill.mjs`.
- `git diff --check` passed for task-owned files.
- The first rebuild attempt used the historical `rawrarr` service name, which does not exist in the current Compose layout. A subsequent default Compose invocation recreated the coordinator without its Tailscale network overlay and hit the already-owned loopback port 7298; this was recovered by applying the existing MacBook Tailscale overlay, which restored the coordinator without changing its Serve settings.
- Rebuilt only the `metadata` service with `--no-deps` under the overlay-aware Compose files. The container is healthy, port 7297 returns HTTP 200, and its binary contains the new `Artwork cache complete` UI string.
- The existing Chrome tab could not be re-read after its refresh because the very large Movies accessibility tree repeatedly timed out. Chrome had shown the old UI before deployment; final rendered UI remains unverified.

## Final state

The source UI and focused regression test are updated and the persistent metadata container runs the rebuilt image. Confirm the final rendered state in Chrome after a normal reload.
