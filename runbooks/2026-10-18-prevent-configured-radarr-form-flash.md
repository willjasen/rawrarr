# Prevent a flash of configured Radarr form

## Objective

Prevent the Movies connection form from flashing while the browser loads saved Radarr settings. Apply the same initial hidden state to the Sonarr form on TV.

## Initial state

Media connection cards were hidden after an asynchronous config read returned `configured: true`, but their markup started visible. Read [the configured Arr form runbook](2026-10-18-hide-configured-arr-forms.md) for the preceding visibility change. Preserve unrelated workspace changes.

## Actions

- Mark both media-tab cards hidden in their initial HTML.
- Keep the existing configured-state response responsible for showing an unconfigured service's form.
- Add a browser assertion that the Movies form starts hidden and static checks for both initial hidden attributes.

## Verification

The Radarr/Sonarr browser flow and the TV/settings static checks passed (2 selected tests, 0 failures); Node syntax and scoped diff checks passed. The broader paired suite ran twice and each time its new visibility flow passed, but an unrelated pre-existing dinosaur-scenery assertion failed (the scene set lacked `volcano`). Rebuilt roarlay in the existing `rawrarr-local` project; it is healthy, local port 7297 and the public HTTPS health route return HTTP 200, Tailscale Serve routes are unchanged, and the running binary contains both media-card markup IDs.

## Recovery

Remove the initial `hidden` attributes to restore the prior rendering behavior.
