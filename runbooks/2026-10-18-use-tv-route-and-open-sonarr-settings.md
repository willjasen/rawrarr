# Use the TV route and open Sonarr settings

## Objective and initial state

The coordinator's TV navigation displayed the right label but wrote `#series`; the TV page only described Sonarr setup, leaving the entry point to the existing settings form unclear. Preserve the internal series view and Sonarr API names while making `#tv` canonical.

## Changes

- Map `#tv` to the existing TV/series view and write `#tv` when navigating there. Upgrade old `#series` links to `#tv` on load.
- Add a TV-page button that opens Settings at the Sonarr URL/API-key form.
- Extend Chromium coverage for both route forms, the canonical hash, the TV-to-settings action, and encrypted Sonarr save through the direct metadata route.

## Verification

The isolated Chromium tests passed: the coordinator Sonarr save flow, direct-route credential protection, `#tv` loading, legacy `#series` upgrade, and TV-page navigation to the Sonarr form. The coordinator workspace static tests passed (12 total checks across both test files). `node --check` and `git diff --check` passed.

Deployed the UI to `rawrarr-local` roarlay. The container is healthy, Tailscale Serve still maps `/` to `http://127.0.0.1:8080` and `/direct-api` to `http://127.0.0.1:8082`, HTTPS health returned 200, and the served browser module contains the new canonical route and Sonarr settings entry point. The deployment lock was released after verification.

A real Sonarr endpoint and credentials were not entered, so live Sonarr connectivity and encrypted persistence against the user's instance remain for the user to confirm in Settings.
