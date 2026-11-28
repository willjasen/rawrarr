# Public Roarlay dinosaur favicon

Issue: https://github.com/willjasen/rawrarr/issues/512

## Objective and initial state

Show the Rawrarr dinosaur as the favicon for desktop Chrome on `roarlay.rawrarr.net`. The live `/auth` page advertises the existing SVG and ICO icon routes, but unauthenticated GET requests to both routes return HTTP 401. The regular logo asset returns HTTP 200. The current repository source already registers both favicon routes as top-level public routes.

The task began from clean `origin/main` (`484cafe0bc3268216760768125da307b76b01933`) in branch `codex/public-landing-dinosaur-favicon`, in its own worktree. No persistent application state or subscriber data is involved.

## Changes

- Investigated the live public Roarlay HTML and unauthenticated responses. `/auth` returns HTTP 200 and includes both favicon links; `/assets/rawrarr-favicon.svg` and `/favicon.ico` return HTTP 401, while `/assets/rawrarr-logo.png` returns HTTP 200.
- Confirmed current `origin/main` and the deployed revision source both contain handlers and route registrations for the SVG and ICO paths. The deployed `rawrarr-prod-roarlay` container reports revision `566310acb1daac6fe4c0307895a72070c0b230a1` and healthy status.
- From inside that container, unauthenticated GET requests to `/assets/rawrarr-favicon.svg` and `/favicon.ico` return HTTP 401; `/assets/rawrarr-logo.png` returns HTTP 200. The public hostname returns the same statuses for those assets.
- No service configuration or runtime state was changed in this investigation. No tests were run because no code change was made.

Persistent state and protocols are unaffected. The remaining work is to identify why the live server denies these registered public routes, then correct and verify the public deployment. The runtime image is revision `566310a`; `origin/main` is `484cafe0bc3268216760768125da307b76b01933`.

## Final state

The cause is narrowed to live application routing or access behavior: the checked-in handlers are present in both current main and the image's reported source revision, but unauthenticated requests receive 401. The dinosaur favicon remains unavailable in Chrome until this is resolved. Issue #512 remains open.
