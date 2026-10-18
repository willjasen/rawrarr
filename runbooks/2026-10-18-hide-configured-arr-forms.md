# Hide configured Arr connection forms

## Objective

Hide the Radarr and Sonarr connection cards on Movies and TV once each service is configured, while preserving Settings as the place to update or clear saved connection details.

## Initial state

Movies and TV always rendered their connection cards, even when the metadata node reported saved settings. Settings already provided separate Radarr and Sonarr forms with saved-state confirmations. The repository contains extensive unrelated staged, modified, and untracked work; preserve it.

## Actions

- Tie each media-tab connection card's visibility to the configured flag returned by its direct metadata settings request.
- Restore the card after its connection is cleared in Settings.
- Extend browser and static regression checks for configured hiding and post-clear reappearance.
- Preserve the coordinator diagnostic listener on 7298 and publish the same loopback-only web listener on the required local port 7297.

## Verification

The focused coordinator browser and static tests passed (19 tests, 0 failures). Node syntax checks and scoped `git diff --check` passed. Rebuilt roarlay and the Tailscale namespace in the existing `rawrarr-local` Compose project. Both containers are healthy/running; `http://127.0.0.1:7297/health` and the public HTTPS health route return HTTP 200. Tailscale Serve still maps `/` to `http://127.0.0.1:8080` and `/direct-api` to `http://127.0.0.1:8082`; the 7298 diagnostic mapping remains. The running binary contains the Radarr and Sonarr media-card IDs.

## Recovery

Restore the media cards by removing the configured-state visibility assignment; saved Arr settings and Settings forms are independent of this presentation behavior.
