# Manage Radarr and Sonarr from Settings

## Objective

Give Radarr and Sonarr consistent, readable connection forms in Settings and confirm when each service's details have been saved.

## Initial state

Radarr and Sonarr forms were available only on Movies and TV. Settings did not expose those forms or display saved connection state. The current repository contains extensive unrelated staged/unstaged workspace changes; preserve them.

## Actions

- Add stacked, responsive Radarr and Sonarr forms to Settings while retaining the existing media-tab forms.
- Keep both copies synchronized with the selected metadata node's age-encrypted settings. API keys are cleared from both form copies after save and are never displayed in the saved-state box.
- Show a confirmation box for each configured service with its saved URL and confirmation that the API key is stored in the age-encrypted config.
- Add browser and static regression coverage for settings-page saves, saved URL readback, API-key secrecy, and the retained Movies/TV forms.

## Verification

Focused coordinator Chromium and static tests passed: 16 tests, 0 failures. `node --check` and `git diff --check` passed. Rebuilt the existing `rawrarr-local` roarlay service; it is healthy, HTTPS `/health` returned HTTP 200, and Tailscale Serve still routes `/` to `http://127.0.0.1:8080` and `/direct-api` to `http://127.0.0.1:8082`. Confirmed the running binary contains the Radarr and Sonarr saved-connection messages. The deployment marker was released after verification.

## Recovery

The connection forms on Movies and TV remain available. Reverting the Settings forms leaves those original forms and the encrypted Radarr/Sonarr config intact.
