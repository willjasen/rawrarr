# Fix large TV artwork pipe deadlocks

## Objective and initial state

Adventure Time and other TV posters stayed blank after the age argument fix. The TV page created poster requests, but several remained pending while a smaller subset rendered. Redis and the metadata service were configured and healthy.

Related runbooks: [TV artwork decryption recovery](2026-10-18-fix-tv-artwork-redis.md), [TV artwork rendering](2026-10-18-show-sonarr-series-artwork.md), and [metadata Redis cache](2026-10-18-implement-metadata-redis-cache.md).

## Findings and actions

- The shared age subprocess helper wrote the full ciphertext to child stdin before draining child stdout. Large images can fill both pipes and leave both processes blocked.
- Changed the helper to write stdin on a scoped worker thread while `wait_with_output` drains stdout/stderr. Applied the same fix to encryption and decryption.
- Expanded the encrypted artwork round-trip fixture to 512 KiB so it exceeds typical pipe capacity and exercises both directions.
- Deploy only the MacBook metadata service under its per-container lock. The shared TV metadata tree is authoritative; do not delete it to address this pipe issue.

## Verification

- The encrypted artwork round-trip regression passed with a 512 KiB poster fixture.
- Rebuilt only the MacBook metadata service. It is healthy, Redis returns `PONG`, and direct health reports the cache configured and reachable.
- Adventure Time (TVDB 152831), A Christmas Carol (371253), Alice in Wonderland (290901), and All That (75208) all returned HTTP 200. Browser image dimensions were 680 × 1000, and the posters are visible on the TV page.
- All 228 series rows have poster URLs. Of the 27 images loaded before scrolling, none failed; the remaining images are still lazy-loaded.
- Redis recorded 590 keyspace hits and 154 misses after the page load. The metadata container returned to 33 PIDs after restart, down from 363 observed while the old subprocesses were blocked.
- Tailscale Serve still routes `/` to `127.0.0.1:8080` and `/direct-api` to `127.0.0.1:8082`.

## Recovery

Rollback only the task-owned subprocess helper and regression fixture changes, then rebuild the MacBook metadata service. Existing ciphertext does not need migration or rescraping.
