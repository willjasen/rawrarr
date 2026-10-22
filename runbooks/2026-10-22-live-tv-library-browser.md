# Reuse a live TV style browser for Movies and TV

## Objective and initial state

Make the coordinator Movies and TV libraries feel like media browsing pages while keeping one reusable layout for both services. Both pages used poster thumbnails in dense tables. Related work: [cached library text](2026-10-22-cache-library-text-in-browser.md), [shared artwork progress](2026-10-22-shared-arr-scrape-progress.md), and [reachable Arr detail links](2026-10-18-coordinator-arr-detail-links.md).

## Actions

- Replace the separate Movies and TV tables with the same responsive poster-card browser.
- Add shared title search, availability/monitoring filters, result counts, and empty states. Keep service-specific details: movie file availability and TV episode counts.
- Preserve lazy poster loading and reachable Radarr/Sonarr links in card titles.
- Update the browser regression coverage for card rendering, search filtering, and the shared link behavior.

## Verification

Pending implementation checks.

## Deployment and remaining work

The coordinator UI is the target. Deployment and authenticated browser read-back remain pending.
