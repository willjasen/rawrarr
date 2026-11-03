# Media service settings dialog

## Scope

Compact the Settings Radarr and Sonarr connection summaries and move configuration into a popup. Add a Test action that sends the supplied URL and credentials to the selected metadata node for validation, without calling Radarr or Sonarr from the browser. Existing saved configuration remains in the node's age-encrypted config.

## Tracker

Created [GitHub issue #270](https://github.com/willjasen/rawrarr/issues/270): **Compact media service settings with node-side connection testing**.

Acceptance criteria:
- Settings shows compact Radarr and Sonarr summaries with configured state.
- A configure/change button opens a dialog for URL/API key and optional Radarr UI credentials.
- The dialog Test action validates from the selected metadata node and does not save.
- Save and clear continue to use the node's age-encrypted configuration.
- Credentials do not pass through roarlay or remain displayed after saving.

## Compatibility and rollout

No persistent schema, encrypted config format, or protocol compatibility change. The new authenticated metadata-node test routes are additive. Existing nodes without these routes cannot perform the new test action until updated; saving continues to validate and save using the existing routes. Deploy the coordinator UI and metadata application together when adopting this feature.

## Verification

- Added compact Radarr and Sonarr status cards with dialogs for configuration.
- Added authenticated metadata-node POST tests for Radarr and Sonarr. These call the service status API from the node and do not save settings.
- Added source and Chromium regressions for card states, dialog flows, node-side tests, saves, clears, and credential cleanup. The full isolated OrbStack regression suite passed: 142 Node tests, all Playwright tests, and 113 Rust tests.
- `git diff --check` passed. No persistent data migration is required.
