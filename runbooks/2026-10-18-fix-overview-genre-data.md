# Fix Overview genre data

## Objective

Populate the Overview genre charts from the Radarr and Sonarr libraries.

## Initial state

The live Overview showed 1,945 Radarr movies but no genre data. The readable web-export source already mapped genres, while the deployed minified browser bundle had drifted and discarded the genre property. Initial metadata-node selection also did not load either Arr configuration when Overview was selected, so Sonarr statistics and genres could remain unloaded.

## Actions

- Rebuilt the minified metadata-export bundle from its source so decrypted Radarr library records retain their genre arrays.
- Loaded Radarr and Sonarr settings when a metadata node is selected on Overview, which allows the existing Sonarr loader to fill its chart when configured.
- Added regression checks for genre preservation and Overview loader initialization.

## Verification

`npm run build` in `rust-arr/web-export` regenerated the bundle. `node --check` passed. The genre-specific checks in `test_rawrarr_coordinator_workspace.mjs` passed; the full suite had 14/18 passing and the same four unrelated existing failures from stale coordinator markup/route assertions. Rebuilt `roarlay` in `rawrarr-local`; it became healthy, local and tailnet health both returned HTTP 200, and Tailscale Serve JSON retained the `/` proxy to `http://127.0.0.1:8080` and `/direct-api` proxy to `http://127.0.0.1:8082`. After reload, the authenticated Overview visibly showed both pie charts: Radarr had 11 slices including `Other 607`, and Sonarr had 11 slices including `Other 253`. The deployment marker was removed.

## Recovery

Revert this runbook and index entry, the Overview selection change in `rust-arr/src/coordinator-workspace.mjs`, and the generated `rust-arr/src/assets/metadata-export.js`. No media metadata or library records are modified.
