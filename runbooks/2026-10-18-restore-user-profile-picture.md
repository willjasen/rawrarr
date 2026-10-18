# Restore the coordinator user profile

## Objective and initial state

Show the account name selected during passkey setup and restore a generated profile picture in the signed-in coordinator workspace. The coordinator stores the account display name in PostgreSQL, but its Settings page had no profile view or account endpoint. The checkout contained extensive unrelated changes; this task was limited to coordinator profile UI and data access.

## Actions

- Added an authenticated coordinator workspace endpoint that returns only the signed-in account's display name.
- Replaced the Settings placeholder with a User profile card. The browser generates a deterministic geometric SVG picture locally from the saved display name; no third-party avatar service or subscriber-node data is involved.
- Extended the coordinator workspace regression test.

## Verification

- The focused coordinator workspace Node suite passed (7 tests); `git diff --check` passed.
- The first Compose attempt omitted the Tailscale namespace overlay and could not bind 7298, which is owned by `rawrarr-macbook-tailscale`. Re-ran the coordinator-only deployment with the established `docker-compose.tailscale-macbook.yml` overlay; no other service or volume was recreated.
- Docker built the Rust release image successfully. The coordinator is healthy; local `http://127.0.0.1:7298/health` and tailnet `/health` both returned HTTP 200. Tailscale Serve read-back confirms `/` still proxies to `http://127.0.0.1:8080` and the `/direct-api` handler remains configured.
- An unauthenticated request to the new profile endpoint returned HTTP 401 as expected. The authenticated profile response was not exercised against the live account; the browser UI regression and SQL ownership scoping were checked statically.

## Final state and remaining work

- Deployed to the coordinator on the MacBook. The signed-in User profile view displays the stored passkey setup name and a deterministic generated picture created locally in the browser.
- No schema or user data changed. Live authenticated profile rendering remains unverified without completing a browser sign-in.
