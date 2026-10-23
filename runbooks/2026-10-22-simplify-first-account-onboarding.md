# Simplify first-account onboarding

## Objective and initial state

After passkey registration, complete age ID and user signing-key setup, then open Devices. Keep node Docker Compose generation available from Devices for when the account owner chooses to add a node. Initial inspection found first registration redirected to `/setup/node`, where Compose was a required third step; Devices had a generic zero-node status.

GitHub issue creation was attempted with `gh issue list` and `gh issue create`, but the GitHub API was unreachable (`error connecting to api.github.com`). Proposed issue: **Route first account setup to Devices before node deployment**. Acceptance criteria: first account lands on Devices after required identity setup; zero-node Devices view offers an Add node link; Compose generation remains accessible only on explicit choice.

Relevant prior context: [Subscriber node onboarding](2026-10-18-subscriber-node-onboarding.md), [Implement browser mini metadata export](2026-10-18-implement-browser-mini-metadata-export.md).

## Actions

- Created isolated task branch `codex/onboarding-compose-devices` from `main` in a dedicated worktree.
- Changed first coordinator passkey registration to redirect to `/#devices`; the workspace now reads hash changes so the Devices tab is selected after navigation.
- Added a first-node Devices empty state linking to the existing `/setup/node` subscriber Compose generator.
- Changed the age and signing-key page's completed state to offer Continue to Devices and an explicit Set up a node now action. Preserved Compose generator and security backup gates.
- Updated the focused browser test and its index.

## Verification

Passed `node --check` for both changed JavaScript modules, `git diff --check`, and `python3 -m unittest scripts.tests.test_ui_bundle` (6 tests). The focused Chromium browser test was attempted but could not start because `/usr/local/lib/node_modules/playwright-core/index.mjs` is missing from this host. No Rust or live sign-up check was run.

## Recovery and remaining work

Revert only this task's commits/files to restore mandatory Compose setup after first registration. GitHub issue creation remains to be retried when API access is available. No deployment requested or performed.
