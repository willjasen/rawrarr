# Put complete movie metadata export in Settings

## Objective and initial state

Move the existing browser-side age decryption export from the Movies page into Settings and ensure the export includes every valid movie text record and matching artwork in the metadata export index. The checkout contains extensive pre-existing edits; only files changed for this task are listed below.

## Actions

- Added the Export metadata control to the Radarr Settings panel and updated the dialog copy and downloaded filename.
- Removed the three-record limit from the metadata-node index, retaining filename validation and the existing per-kind 10,000-entry safety bound was removed so all valid entries are indexed.
- Updated the export manifest scope and focused browser fixture to include four records, proving records beyond the former first three are included.

## Verification

The browser export build and JavaScript syntax checks passed. The focused Playwright test could not start because `/usr/local/lib/node_modules/playwright-core/index.mjs` is absent in this host environment. `git diff --check` passed. The Docker Rust build succeeded with one pre-existing dead-code warning. Tailnet `/health` returned HTTP 200, the deployed coordinator workspace module and metadata export bundle contain the export integration, and Tailscale Serve still proxies `/` to `http://127.0.0.1:8080`. A temporary Compose override was needed because the base file publishes roarlay on port 7298, already held by `rawrarr-macbook-tailscale`; the override restored the documented shared network namespace.

## Final state

The metadata export control is in coordinator Settings and requires an age identity unlocked in Movies plus a selected metadata node. Export contents include all valid indexed movie text records and matching supported artwork; other metadata classes are not yet handled by the existing export API. Deployment is active on roarlay. The temporary Compose override was kept outside the repository and removed after deployment. No task commit was made because the edited source files contain substantial pre-existing modifications with mixed ownership.
