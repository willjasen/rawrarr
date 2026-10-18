# Share the age private ID across Rawrarr browser tabs

## Objective and initial state

Allow a newly opened Rawrarr tab in the same browser profile to load the encrypted movie library after the user has unlocked it in another tab. The key was stored only in `sessionStorage`, so each tab had to be unlocked separately.

## Actions

- Changed age private ID storage to origin-scoped `localStorage`, with migration from the old tab-scoped value when the coordinator page reloads.
- Added a storage event listener so other open Rawrarr tabs unlock when the key is saved and clear their decrypted view when it is removed.
- Updated clear and sign-out actions to remove the key from browser storage. Updated the UI and security documentation to describe profile-scoped persistence.
- Updated the coordinator workspace test for storage, cross-tab events, and clearing.

## Verification

The focused coordinator workspace test passes 6/6. The coordinator image rebuilt and the Compose coordinator is healthy; its HTTPS health endpoint returns HTTP 200, and Tailscale Serve still forwards the root path to `http://127.0.0.1:8080`. Reloading the original tab migrated its existing session-scoped key into the browser profile. The new Chrome tab received the storage event and displayed all 1,941 decrypted movie entries, with posters and statistics. The key was never read into agent output or recorded. Never copy the private ID into logs or runbook text.

## Recovery

The user can remove the stored key from any Rawrarr tab with **Clear private ID**. Reverting the client change restores tab-scoped storage; existing profile-scoped data should then be cleared from Rawrarr before rollback.
