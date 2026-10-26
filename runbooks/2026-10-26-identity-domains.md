# Define user, device, group, and app identity domains

Issue [#95](https://github.com/willjasen/rawrarr/issues/95).

## Objective

Document ownership, creation, storage, purpose, recovery, rotation, and implementation state for Rawrarr's user, device, group, and managed-app identity domains. Keep age confidentiality keys distinct from passkey authentication and ML-DSA signing keys.

## Initial state

Started from fetched `origin/main` at `6159e04` in dedicated worktree `/Users/willjasen/.codex/worktrees/issue-95-identity-domains`, branch `codex/issue-95-identity-domains`. The repository already documented user and device age identities and device signing separately, while the agreed group/app lifecycle existed primarily in the issue description. `docs/DESIGN.md` also had an exploratory multi-person recipient model that could be mistaken for the agreed key-domain design.

## Actions

- Added `docs/identity-domains.md` as the consolidated source of truth, with a current-versus-planned matrix and separate lifecycle guidance.
- Linked the document from the documentation index, user guide, and device-signing design.
- Marked the multi-person cluster section in `docs/DESIGN.md` as exploratory and directed readers to the agreed identity model.

## Verification

- Manually checked identity claims against `docs/device-signing-design.md`, `docs/user-guide.md`, `runbooks/2026-10-22-node-and-app-age-identities.md`, and `runbooks/2026-10-22-complete-device-signing.md`.
- Confirmed group/app key distribution and rotation are explicitly labeled planned; current shared metadata encryption remains described as using the owner age recipient.
- `git diff --check` passed. No automated tests were run because this task changes documentation only.

## Final state and remaining work

The four identity domains and key-purpose separation are documented, with current behavior distinguished from planned behavior. Group lifecycle, per-app identities, messaging, and app migration remain implementation work tracked by issues #109–#112. No runtime change or deployment was made.
