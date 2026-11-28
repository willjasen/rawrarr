# Public Roarlay dinosaur favicon

Issue: https://github.com/willjasen/rawrarr/issues/512

## Objective and initial state

Show the Rawrarr dinosaur as the favicon for desktop Chrome on `roarlay.rawrarr.net`. The live `/auth` page advertises the existing SVG and ICO icon routes, but unauthenticated GET requests to both routes return HTTP 401. The regular logo asset returns HTTP 200. The current repository source already registers both favicon routes as top-level public routes.

The task began from clean `origin/main` (`484cafe0bc3268216760768125da307b76b01933`) in branch `codex/public-landing-dinosaur-favicon`, in its own worktree. No persistent application state or subscriber data is involved.

## Changes

- Investigated the live public Roarlay HTML and unauthenticated responses. `/auth` returns HTTP 200 and includes both favicon links; `/assets/rawrarr-favicon.svg` and `/favicon.ico` return HTTP 401, while `/assets/rawrarr-logo.png` returns HTTP 200.
- Confirmed current `origin/main` source registers both favicon routes at the top level, outside the session-protected router.

This appears to be a mismatch between the public deployment and current source. No persistent-state or protocol compatibility impact is indicated. Deployment changes require using the published `main` image and should be handled only after identifying the running public service revision.

## Final state

Pending.
