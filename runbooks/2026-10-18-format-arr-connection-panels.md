# Format Arr connection panels on Movies and TV

## Objective and initial state

The TV page showed a large button leading away from the page to enter Sonarr details. The coordinator had no matching Radarr connection form on Movies, even though the metadata node already exposed Radarr settings endpoints. Make service setup consistent and easy to scan on both media pages.

## Changes

- Added styled, labeled, full-width URL and password fields with save, clear, and status feedback on Movies (Radarr) and TV (Sonarr).
- Kept API keys on the browser-to-selected-metadata direct route; added the protected Radarr settings route to that direct API.
- Read the age key from the current tab's in-memory state so setup works after unlocking from Movies. Keep the field empty after save and only load the saved URL.
- Added browser coverage for both media forms, save/read/clear flows, direct-route secret boundaries, and canonical TV navigation; added static checks for the route and shared form styling.

## Verification

The focused Chromium tests passed: 12 tests covered the shared form layout, direct-only Radarr/Sonarr save/read/clear flows, age unlock, secret-boundary checks, and `#tv` navigation. The Rust encrypted-settings preservation test passed (`sonarr_and_radarr_updates_preserve_each_other_and_filename_key`). JavaScript syntax checks and `git diff --check` passed.

Rebuilt metadata and roarlay in the existing `rawrarr-local` Compose project. Metadata and roarlay are healthy; `/direct-api` reaches metadata health (HTTP 204), public HTTPS health returns 200, and Tailscale Serve still routes `/` to `http://127.0.0.1:8080` and `/direct-api` to `http://127.0.0.1:8082`. Metadata briefly restarted while roarlay was concurrently being deployed and became healthy after roarlay returned. Both deployment locks were released after verification.

No live Radarr or Sonarr credentials were submitted during the browser fixture tests. The user can now enter each service URL/API key from its media tab after unlocking the age ID in Movies.
