# Create the Rawrarr product design guide

## Objective and initial state

The project had technical architecture, product direction, and a user guide, but no dedicated living record of how the app should behave and present information. The user asked to keep UI/product decisions current as they recommend changes.

## Actions

- Added `docs/DESIGN.md` to record user-facing design goals, onboarding, age identity display, identity-derived logo, workspace, Radarr, scanning, device sharing, and accessibility expectations.
- Linked the guide from the documentation index and runbook index.
- Explicitly distinguished product decisions from implementation status and technical architecture.

## Verification and remaining work

Relative Markdown targets were validated in 121 files, and `git diff --check` passed. No application behavior or runtime configuration changed. Future accepted UI/product recommendations should update `docs/DESIGN.md` in the same task.
