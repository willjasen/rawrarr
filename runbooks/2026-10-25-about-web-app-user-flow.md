# Describe the current web app on About

## Objective

Rewrite the coordinator About page to explain the web app's current user flow without presenting future identity-domain or container-provisioning behavior as available. Track the separately described identity domains and Radarr/Sonarr provisioning workflow in distinct GitHub issues.

## Initial state

- Started from the latest fetched `origin/main` in dedicated worktree `/Users/willjasen/.codex/worktrees/about-web-app-user-flow`, branch `codex/about-web-app-user-flow`.
- The existing About page described general encrypted sharing and future file sharing. Current browser workflows are documented in `docs/user-guide.md` and implemented in `rust-arr/src/coordinator_workspace.html` and `rust-arr/src/coordinator-workspace.mjs`.
- Existing issues #37 and #40 cover portions of device signing and shared-data trust; #34 covers subscriber installation. They do not describe the two complete workflows Will requested.

## Actions

- Created [issue #95](https://github.com/willjasen/rawrarr/issues/95) for the user, device, and group identity domains, linked to existing signing issues #37 and #40.
- Created [issue #96](https://github.com/willjasen/rawrarr/issues/96) for capturing supported settings from an external Radarr/Sonarr and provisioning a Rawrarr-managed instance, linked to subscriber onboarding issue #34.
- Rewrote the About page to describe passkey sign-in, selecting an approved fossil, connecting existing Radarr/Sonarr, browsing Movies/TV, browser-side age unlock, device routes, and the scoped relay.

## Verification

- Source review confirmed the browser workflow statements against the current coordinator workspace, its JavaScript, and `docs/user-guide.md`.
- `git diff --check` passed.
- GitHub read-back confirmed issues #95 and #96 are open with the requested separate scopes.
- No automated tests were run; this change updates About copy only.

## Final state

The About copy now describes the current web app. The three-domain identity lifecycle and external Arr capture/provisioning remain future work tracked by #95 and #96. Changes are committed on the task branch; no deployment was performed.
