# Reuse a live TV style browser for Movies and TV

## Objective and initial state

Make the coordinator Movies and TV libraries feel like media browsing pages while keeping one reusable layout for both services. Both pages used poster thumbnails in dense tables. Related work: [cached library text](2026-10-22-cache-library-text-in-browser.md), [shared artwork progress](2026-10-22-shared-arr-scrape-progress.md), and [reachable Arr detail links](2026-10-18-coordinator-arr-detail-links.md).

## Actions

- Replace the separate Movies and TV tables with the same responsive poster-card browser.
- Add shared title search, availability/monitoring filters, result counts, and empty states. Keep service-specific details: movie file availability and TV episode counts.
- Preserve lazy poster loading and reachable Radarr/Sonarr links in card titles.
- Update the browser regression coverage for card rendering, search filtering, and the shared link behavior.

## Verification

- `node --check rust-arr/src/coordinator-workspace.mjs`, `node --test scripts/tests/test_rawrarr_library_browser.mjs`, the focused Arr-link source test, and `git diff --check` passed.
- The existing Chromium coordinator test could not start because its hard-coded `/usr/local/lib/node_modules/playwright-core/index.mjs` dependency is absent on this host. Broader coordinator tests also have baseline expectation failures unrelated to this change.
- The authenticated coordinator page rendered 1,945 movie cards and 229 TV series cards with shared search, filters, result counts, and posters. The Movies and TV pages were visually checked in the existing Chrome profile.
- The mini roarlay is healthy with zero restarts. Local and tailnet coordinator health checks returned HTTP 200. Tailscale Serve JSON maps `rawrarr.risk-mermaid.ts.net:443` `/` to `http://127.0.0.1:8080`.
- The deployed coordinator JavaScript asset SHA-256 matches its patched source: `8d80ed3734945118aa04405c18250b3950dbd4d00df25b087ce5c12941b74a29`. The deployment lock was removed after verification.

## Deployment and remaining work

- Committed as `5e404b5` (`feat: unify movie and TV library browsing`). Applied only this commit's UI patch to the mini's newer coordinator source, preserving its unrelated local device and scraping changes. Previous assets are backed up at `~/Library/Application Support/rawrarr/internal-coordinator/backups/pre-5e404b5-live-tv-library/`.
- Rebuilt only `roarlay` in Compose project `rawrarr-internal-mini`; PostgreSQL and the Tailscale sidecar were preserved. Used the standalone `docker-compose.internal-mini.yml` after the base Compose file caused a port/network-mode conflict before changing the container.

To roll back the UI, restore the two saved workspace assets to `internal-coordinator/source/rust-arr/src/` on the mini and rebuild only roarlay under its deployment lock. Preserve PostgreSQL, Tailscale state, and the mini's unrelated source edits.
