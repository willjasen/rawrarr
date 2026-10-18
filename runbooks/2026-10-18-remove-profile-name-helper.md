# Remove the profile name helper text

## Objective and initial state

Remove the redundant “Your name from passkey setup” line from the coordinator Settings profile card while keeping the account display name and generated dinosaur avatar.

## Actions

- Removed the helper sentence from the profile card and added a regression assertion that it stays absent.

## Verification

- `node --test scripts/tests/test_rawrarr_coordinator_workspace.mjs`: 11 passed; `git diff --check` passed.
- Confirmed localhost port 7297 is unoccupied and port 7298 is owned by the existing OrbStack Tailscale namespace. Rebuilt only `roarlay` in `rawrarr-local` using the coordinator and MacBook Tailscale Compose overlays.
- `rawrarr-local-roarlay-1` reports healthy; local and tailnet `/health` both returned HTTP 200. Tailscale Serve still maps `/` to `http://127.0.0.1:8080` and preserves `/direct-api` at `http://127.0.0.1:8082`.
- Unauthenticated access to the profile HTML returned HTTP 401, as expected. A signed-in visual check was not performed.

## Final state and remaining work

- The redundant helper is removed from the deployed profile card. The user's display name and age-ID-derived dinosaur remain.
