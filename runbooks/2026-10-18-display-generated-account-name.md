# Display the generated account name

## Objective and initial state

Display the generated dinosaur-themed account name for existing accounts too. First-passkey enrollment now generates and stores a username/display name, but the workspace profile still read `display_name` from the existing account row, so the previously created account continued to show `willjasen`.

Related work: [generate a dinosaur-themed account name](2026-10-18-generate-dinosaur-account-name.md).

## Changes

- The authenticated workspace profile now derives the displayed account name from its stable account UUID using the same generator as enrollment.
- Stored usernames, account records, and passkey credentials are unchanged; the app profile presents the generated label for both existing and new accounts.
- Updated the coordinator workspace regression assertion.

## Verification

- `node --test scripts/tests/test_rawrarr_coordinator_workspace.mjs` passed all 13 checks.
- The roarlay release image built with the profile endpoint change and the `rawrarr-local-roarlay-1` health status returned `healthy`.
- Read back Tailscale Serve JSON and confirmed `rawrarr.risk-mermaid.ts.net:443` `/` proxies to `http://127.0.0.1:8080`; the HTTPS health route identifies Roarlay.
- Refreshed the existing signed-in Rawrarr tab. The sidebar, Settings profile, and generated dinosaur avatar now display `fern-pterosaur-a1dc`.
- The deployment marker was released after verification, and `git diff --check` passed.

## Final state

Existing accounts now display the same deterministic generated label derived from their account UUID. Their stored username, database row, and passkey credentials were not changed. The roarlay UI is deployed and the visible profile was confirmed in the existing browser session.
