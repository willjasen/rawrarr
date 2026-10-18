# Hide completed Sonarr scan messages

## Objective

Hide the scrape-complete and successful series-load messages on the TV view after the Sonarr artwork cache has finished.

## Initial state

The completed TV page continued showing “Sonarr artwork cache complete” and “Loaded … series from Sonarr.” The repository has extensive pre-existing changes; preserve them.

Related procedure: [Sonarr series artwork](2026-10-18-show-sonarr-series-artwork.md).

## Actions

- When the progress endpoint reports an inactive scan with all eligible posters saved, hide the scrape status, progress bar, and successful load message.
- Keep loading and error messages visible, and keep progress details visible while the scan remains incomplete.
- Restrict the progress bar's visible display rule to elements without the HTML `hidden` attribute so completed scans do not leave an empty bar outline.
- Added a focused regression test for completion visibility behavior.

## Verification

- The focused regression test passed; JavaScript syntax and `git diff --check` passed.
- Rebuilt `roarlay` in the existing `rawrarr-local` project. The container is healthy; local and HTTPS health checks return HTTP 200. Tailscale Serve still maps `/` to `http://127.0.0.1:8080` and `/direct-api` to `http://127.0.0.1:8082`.
- The live TV page shows 228 series and poster thumbnails. Browser inspection confirmed the completed scrape status, progress bar (including computed display), and successful series-load message are hidden.
- The repository had extensive unrelated changes before this task. Mixed files remain uncommitted; no push was attempted.

## Recovery

Restore the TV page's completion status display by reverting the completion branch in `coordinator-workspace.mjs`. No metadata or artwork files are changed.
