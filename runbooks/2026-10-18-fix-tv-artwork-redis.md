# Fix TV artwork serving with Redis enabled

## Objective and initial state

Restore TV poster artwork on the coordinator page through the MacBook metadata node while leaving Redis enabled. The TV library loaded, but cached poster reads returned HTTP 500. Related procedures: [TV artwork rendering](2026-10-18-show-sonarr-series-artwork.md) and [metadata Redis cache](2026-10-18-implement-metadata-redis-cache.md).

## Findings and changes

- The age decrypt helper appended the identity file and ciphertext input arguments itself, but artwork loading also supplied `--identity -`. That duplicate/misplaced identity argument made every TV artwork decrypt fail, including the existing encrypted artwork round-trip test.
- Corrected artwork decryption to pass `--decrypt --identity` and let the shared helper append the identity-file path and stdin marker.
- Added safe server-side error logging and Sonarr backfill behavior that retries absent or undecryptable posters while preserving hard storage failures.
- The on-demand TV rescan refreshed 226 existing TV artwork files; text metadata was left intact. No TV metadata deletion was needed.
- Rebuilt only the MacBook `metadata` service under its deployment lock. Redis remained healthy and configured; the authenticated TV page returned successful poster requests and displayed artwork.

## Verification

- Rust artwork tests: 6 passed, including the encrypted artwork round-trip and corrupt-artwork backfill behavior.
- MacBook metadata container healthy; Redis `PING` returned `PONG`; direct health reported the cache configured and reachable.
- Signed-in coordinator TV page loaded 228 series and showed posters. Metadata logs recorded successful HTTP 200 responses for TV poster requests.
- The focused legacy Node Sonarr scrape test still has stale assertions against earlier source layout; it fails before reaching the new assertion. No unrelated test expectations were changed.

## Recovery

The fix is code-only and can be rolled back by reverting the task-owned source changes and rebuilding the MacBook metadata service. Redis entries are disposable; if troubleshooting requires clearing artwork entries, delete only the `rawrarr:artwork:v1:*` prefix. Do not remove the synchronized TV metadata tree: no deletion was needed to restore artwork.
