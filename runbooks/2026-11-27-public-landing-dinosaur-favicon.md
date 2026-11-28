# Public Roarlay dinosaur favicon

Issue: https://github.com/willjasen/rawrarr/issues/512

## Objective and initial state

Show the Rawrarr dinosaur as the favicon for desktop Chrome on `roarlay.rawrarr.net`. The live `/auth` page advertises the existing SVG and ICO icon routes, but unauthenticated GET requests to both routes return HTTP 401. The regular logo asset returns HTTP 200. Initial inspection found route handlers only in the subscriber-node HTTP router; `main()` starts `coordinator::router()` and returns early for the coordinator role, so those routes were never registered on public Roarlay.

The original investigation began from clean `origin/main` (`484cafe0bc3268216760768125da307b76b01933`) in a dedicated worktree. After that investigation was merged, implementation continued in the same worktree on a new branch from current `origin/main` (`f62fa8e8c25f1f969c1314b82401e319e609e832`). No persistent application state or subscriber data is involved.

## Changes

- Investigated the live public Roarlay HTML and unauthenticated responses. `/auth` returns HTTP 200 and includes both favicon links; `/assets/rawrarr-favicon.svg` and `/favicon.ico` return HTTP 401, while `/assets/rawrarr-logo.png` returns HTTP 200.
- The deployed `rawrarr-prod-roarlay` container reports revision `566310acb1daac6fe4c0307895a72070c0b230a1` and healthy status. That revision contains the favicon handlers and route registrations in `main.rs`, but those are on the subscriber-node router only.
- From inside that container, unauthenticated GET requests to `/assets/rawrarr-favicon.svg` and `/favicon.ico` return HTTP 401; `/assets/rawrarr-logo.png` returns HTTP 200. The public hostname returns the same statuses for those assets.
- Confirmed `main()` starts `coordinator::router()` and exits before building the subscriber-node router in `main.rs`. The coordinator's public router served the HTML and wordmark but had no favicon route registrations; unknown paths fell through to the coordinator auth guard and returned 401.
- Added both icon routes to the coordinator public router and reused the embedded handlers. Made the handlers `pub(crate)` so both the coordinator and node routers share the same asset implementation and content types.
- Extended the favicon regression to assert that both paths are registered in the coordinator public router, and updated its index entry.

This static route change does not affect persistent state, database schemas, subscriber nodes, or protocols. Existing deployments adopt it when the published coordinator image is deployed to the public Roarlay service on do3. Rollback is to redeploy the previous published image; no data migration is needed.

## Final state

The focused favicon regression passed in the isolated Chromium test container (3 tests, 0 failures). The first direct invocation lacked the test runner's Playwright module alias and failed during module resolution; adding the same alias the runner normally creates allowed the test to pass. The temporary container was removed.

GitHub Actions publish run [46091210620](https://github.com/willjasen/rawrarr/actions/runs/46091210620) completed successfully for both architectures. The do3 published-image updater deployed revision `6ec609ddc8b451307026c8c857c1ed47fef5fd1b` to the public and internal coordinators. Both containers report healthy and `/api/version` reports the deployed revision; the updater finished inactive and released both deployment locks. No application data or service configuration changed.

Live public verification: `/auth` returns HTTP 200; `/assets/rawrarr-favicon.svg` returns HTTP 200 with `image/svg+xml`; `/favicon.ico` returns HTTP 200 with `image/x-icon`; and `/health` returns HTTP 200. Issue #512 is resolved and can be closed.
