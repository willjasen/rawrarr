# Node age identities and avatars

## Objective and initial state

Add persistent age identities to subscriber startup and use their public recipients to generate node avatars with the existing deterministic coordinator avatar renderer. The existing user age identity remains independent. Deployment target is the internal MacBook fossil subscriber; coordinator UI and renderer routes deploy to Mac mini roarlay. Existing local edits in the main checkout are unrelated and remain untouched.

## Implementation and verification

- Subscriber startup creates `/data/node-age-identity` when missing and retains it on subsequent starts. It is independent of `/data/age-identity`, the user's vault identity.
- An authenticated subscriber route exposes only the node identity's public age recipient. Roarlay relays this owner-scoped read through the existing allowlist.
- The Devices view hashes each reachable node recipient with SHA-256 and requests the same coordinator-rendered JPEG XL/PNG image used for the user profile. A session cache avoids repeated recipient requests while cards refresh.
- Added `test_rawrarr_node_avatar.mjs`; all three test cases passed. The focused node identity Rust tests passed (2/2), and the coordinator relay allowlist test passed (1/1). Full Rust suite passed (66/66). JavaScript syntax checks and `git diff --check` passed.
- An earlier run of the full legacy coordinator workspace Node test file has existing failures before the new avatar assertions (for example, stale navigation/status expectations); the new standalone avatar tests pass.

## Deployment and recovery

- Deployed commit `af90ce4` to the existing internal `roarlay` and `fossil` app containers. Coordinator source was staged under a task-specific build directory on the Mac mini; the existing `github-source` checkout was not changed. The Mac mini Docker credential-helper configuration initially prevented a build; an isolated Docker config retained its Compose plugin and context while omitting the unavailable keychain helper.
- `roarlay` is healthy with zero restarts. Tailscale Serve read-back confirms `rawrarr.risk-mermaid.ts.net:443` `/` proxies to `http://127.0.0.1:8080`; the HTTPS health page responds and identifies Roarlay. Both new coordinator JavaScript assets return HTTP 200.
- `fossil` is healthy with zero restarts on the image built from this commit. `/data/node-age-identity` exists with mode `600` and its existing 2084-byte content was retained through container recreation. The node recipient endpoint returns HTTP 401 without a user session, as expected for its authentication requirement.
- Deployment locks were used for their respective services and released after verification. PostgreSQL, Tailscale state, `/data`, and `/shared` were preserved; no browser windows were changed.
- The node card/avatar appearance was not visually verified in an authenticated browser session. Sign in to the internal UI and open Devices to confirm the owner-visible cards and generated images in the browser.
