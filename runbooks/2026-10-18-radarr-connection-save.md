# Connect Radarr before saving encrypted credentials

## Objective and initial state

Ensure a user can enter a Radarr URL and API key, confirm that the endpoint accepts them, and save them to the synchronized age-encrypted configuration. The UI already provided the form and `PUT /api/settings/radarr` already wrote `config.aged`, but it accepted syntactically valid settings without connecting first. Related context: [encrypted Radarr settings](2026-10-17-rawrarr-age-encrypted-config.md), [Radarr UI settings](2026-10-17-rawrarr-radarr-ui-config.md).

## Actions

- Added a Radarr status request to the save path. Settings are written only after the endpoint responds successfully and identifies itself as Radarr; successful credentials continue to apply immediately.
- Added a disposable local Radarr HTTP mock to the configuration integration test, including an unreachable endpoint check and a successful authenticated status check.
- Updated the user guide to state that connection is verified before encrypted persistence.

## Verification

Pending.

## Final state and remaining work

Pending.
