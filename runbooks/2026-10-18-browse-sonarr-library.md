# Browse the Sonarr library on TV

## Objective

Show the configured Sonarr library on the coordinator TV page rather than leaving the initial connection placeholder after saving connection details.

## Initial state

TV contained a Sonarr connection form, metadata-size metric, and a static prompt, but no request or table for Sonarr series. The metadata service already exposes `/api/sonarr/series`, which uses its age-unlocked encrypted connection. Keep the API key on the metadata node and use the browser's direct metadata route.

## Actions

- Replace the static TV prompt with series and episode statistics, a Sonarr series table, status text, and refresh control.
- Fetch normalized series through the selected metadata node's direct `/api/sonarr/series` endpoint after confirming the connection is configured.
- Extend the browser fixture and static checks for rendering data, summaries, and refresh controls.

## Verification

The coordinator Sonarr settings browser suite passed (4 tests, 0 failures), including the configured series response, summary counts, refresh endpoint request, and the case where clearing settings hides the library again. The TV static check passed; Node syntax and scoped `git diff --check` passed. Rebuilt roarlay in the existing `rawrarr-local` Compose project; it is healthy, local port 7297 and the public HTTPS health route return HTTP 200, and Tailscale Serve still maps `/` to `http://127.0.0.1:8080` and `/direct-api` to `http://127.0.0.1:8082`. The live Sonarr API response was not inspected from the authenticated browser session; the page now requests it directly from metadata.

## Recovery

Revert the TV library markup and its `loadSonarrSeries` call. The Sonarr settings saved in the age-encrypted config remain independent.
