# Simplify first-account onboarding

## Objective and initial state

After passkey registration, complete age ID and user signing-key setup, then open Devices. Keep node Docker Compose generation available from Devices for when the account owner chooses to add a node. Initial inspection found first registration redirected to `/setup/node`, where Compose was a required third step; Devices had a generic zero-node status.

Tracked by [issue #57](https://github.com/willjasen/rawrarr/issues/57). Acceptance criteria: first passkey registration opens age ID and signing-key setup; after those backups are saved, the owner can continue to Devices without preparing Compose; a coordinator can start with zero approved nodes; the empty Devices view explains the state and links to subscriber Compose setup.

Relevant prior context: [Subscriber node onboarding](2026-10-18-subscriber-node-onboarding.md), [Implement browser mini metadata export](2026-10-18-implement-browser-mini-metadata-export.md).

## Actions

- Created isolated task branch `codex/onboarding-compose-devices` from `main` in a dedicated worktree.
- Kept first coordinator passkey registration directed to `/setup/node`, so the owner completes age ID and signing-key backup before leaving setup.
- Added a first-node Devices empty state linking to the existing `/setup/node` subscriber Compose generator.
- Changed the age and signing-key page's completed state to offer Continue to Devices and an explicit Set up a node now action. Preserved the Compose generator and backup gates.
- Allowed an empty approved-node registry so a new coordinator can serve an account before any subscriber exists; malformed registries remain rejected.
- Updated the focused browser test and its index.

## Verification

Passed `node --check` for both changed JavaScript modules, `git diff --check`, and `python3 -m unittest scripts.tests.test_ui_bundle` (6 tests). The OrbStack full suite reported 69 Node passes and 25 existing/stale failures; its Chromium age-setup test passed, while Rust tests could not download an uncached crate in offline mode. The focused onboarding browser test passed (1/1) after correcting stale expectations in its harness. Rust registry tests passed (2/2). A live Chromium virtual-passkey signup completed age ID and signing-key setup, reached Devices with zero approved nodes, and displayed the empty state; API requests returned 200 and the browser reported no page errors. The OrbStack app health endpoint returned 200 and `/setup/node` redirected to authentication as expected. The signup data was removed with its disposable test database.

## Recovery and remaining work

Revert only this task's commits/files to restore mandatory Compose setup and require preapproved nodes at coordinator startup. The OrbStack deployment uses only the `rawrarr-local` project's disposable `orbstack-test` database and Headscale/Tailscale test state; the preexisting `rawrarr-local_rawrarr-*` volumes were preserved. The app is left running and healthy on `127.0.0.1:7297` with an empty database for a fresh signup. No internal environment was deployed.
