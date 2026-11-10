# Lidarr integration — issue #288

## Scope and compatibility

This first implementation increment connects an existing Lidarr instance from a selected subscriber node. The node validates the instance, stores its URL and API key in the existing age-encrypted service configuration, and exposes normalized artist and release inventory directly to the owner's browser. The browser uses the existing direct-node route; credentials and node-local music paths are not sent to roarlay.

The encrypted config remains schema v1 and stores Lidarr as an additional service entry, so no migration or protocol change is required. Existing Radarr, Sonarr, and Prowlarr entries are preserved on Lidarr save/clear. The integration is read-only and does not change the connected Lidarr instance.

## Implemented

- Lidarr Settings card with node-side connection test, save, read-back of URL only, clear, and secret-field clearing.
- Direct authenticated node endpoints for settings, live connection status, and a combined artist/release library response. The library includes Lidarr provenance and MusicBrainz identifiers (`foreignArtistId`, `foreignAlbumId`) as stable external IDs.
- Coordinator Music view renders artists and releases using text-only DOM updates and fetches inventory from the selected node.
- Focused Rust normalization/config tests and browser coverage are listed in `scripts/tests/README.md`.

## Still planned

- Phase 2 node-hosted Docker deployment and configuration/library adoption flow. It must preserve external Lidarr data and offer reviewable, explicit cutover and rollback steps.
- Live testing against a real Lidarr instance and deployment to internal/public nodes.

## Verification

No persistent data format or cross-node protocol change was introduced. The isolated test clone passed all three Lidarr Rust tests, the encrypted service-preservation test, both settings-source checks, and all four media-management browser tests. The browser test covered the Lidarr save/test/clear flow, credential-field clearing, artist/release rendering, and 1280×800 plus 375×812 layouts with no horizontal overflow. A live Lidarr instance and the full project suites were not tested.
