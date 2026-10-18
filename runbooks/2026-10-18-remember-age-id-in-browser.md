# Remember the age private ID in this browser

## Objective

Let a user who keeps their browser session signed in opt in to keeping the age private ID in browser storage, while making the higher-security non-persistent choice clear.

## Changes

- Add an unchecked browser-memory opt-in beside the age ID field and a warning to leave it off for the highest security.
- Restore a remembered ID after page reload, use it for metadata and encrypted connection changes without re-entry, and clear it when the user clears the key or signs out.
- Keep the default path in tab memory only; do not send the age ID through the coordinator relay.
- Add browser coverage for non-persistence by default, opt-in reload persistence, and sign-out cleanup.

## Verification

- Focused Chromium and static coordinator checks passed: 14 tests, 0 failures. The browser flow verified opt-in reload persistence, direct-only metadata unlock, and clearing the stored identity on sign-out.
- The roarlay deployment marker was owned by another task during verification. Its container was healthy and Tailscale Serve already routed `/` to `http://127.0.0.1:8080` and `/direct-api` to `http://127.0.0.1:8082`.
- Host port `127.0.0.1:7297` had no listener at the time of inspection. No deployment was attempted because another task held the roarlay marker; the live HTTPS application behavior is therefore not verified for this change.
- The app clears the remembered ID when a roarlay API request returns HTTP 401, and also on explicit sign-out or manual clear.

## Final state

Implemented the optional browser-local storage control and highest-security warning. Deployment to roarlay remains outstanding until its container marker is released; no marker was removed or overwritten.
- After the marker became available, rebuilt `roarlay` in Compose project `rawrarr-local`; container health is `healthy`.
- Confirmed `tailscale serve status --json`: `/` routes to `http://127.0.0.1:8080` and `/direct-api` routes to `http://127.0.0.1:8082`; HTTPS `/health` returned HTTP 200. The running roarlay binary contains the new remember-ID UI and session-expiry cleanup.
- Removed the task-owned roarlay deployment marker after verification.
