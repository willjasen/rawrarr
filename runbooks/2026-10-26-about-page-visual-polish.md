# Polish the About page layout

## Objective

Improve the coordinator About page's visual hierarchy while keeping its description limited to current web app behavior.

## Initial state

- Started from fetched `origin/main` in worktree `/Users/willjasen/.codex/worktrees/about-page-visual-polish`, branch `codex/about-page-visual-polish`.
- The About page used one narrow connection card for all user-flow copy.
- The prior [About user-flow runbook](2026-10-25-about-web-app-user-flow.md) records the current web app claims and their source review.

## Actions

- Replaced the single card with a responsive hero, three workflow cards, and a concise privacy/routing callout.
- Kept the copy focused on passkey sign-in, choosing an approved fossil, connecting an existing Radarr/Sonarr, browsing Movies/TV, and browser-side age unlock.

## Verification

- `node --test scripts/tests/test_rawrarr_coordinator_workspace.mjs` passed all 27 tests.
- `git diff --check` passed.
- Rendered the About section in a local-only preview and visually reviewed the desktop layout. No production page or service was changed.
- No mobile screenshot or live deployment verification was performed.

## Final state

The About layout change is committed on the task branch. The pull request records review and merge state; no deployment has been performed.
