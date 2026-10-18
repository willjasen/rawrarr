# Unlock Movies with loaded age ID

## Objective

Keep Movies and Settings consistent when the age private ID is already available in browser memory. Navigating to Movies should begin decrypting the library instead of leaving a prompt to enter the ID again.

## Initial state

Settings could save/load the browser-held age ID, but navigating to Movies with a selected metadata node did not invoke the library unlock flow. The existing node-selection handler only auto-unlocked when a node was selected. Preserve unrelated workspace changes.

## Actions

- Start the existing unlock flow when navigating to Movies with both a loaded age ID and selected node.
- Extend the Settings age-ID browser flow to navigate to Movies and verify decryption starts without re-entry.

## Verification

The browser test that saves an age ID in Settings, navigates to Movies, verifies successful local decryption, and returns to Settings passed. The related static UI/route test passed as well (2 selected tests, 0 failures); Node syntax checks and scoped `git diff --check` passed. During deployment, the service exposed an unrelated invalid Axum route pattern for profile pictures; changed the route to capture the seed as the final path segment and kept JPEG XL response handling. Roarlay now starts healthy, local port 7297 returns HTTP 200, the public HTTPS health route succeeds, and the Tailscale Serve mappings remain `/` to `http://127.0.0.1:8080` and `/direct-api` to `http://127.0.0.1:8082`.

## Recovery

Remove the Movies navigation call to `unlockMetadata`; users will again need to click Unlock after adding an age ID while a node is already selected.
