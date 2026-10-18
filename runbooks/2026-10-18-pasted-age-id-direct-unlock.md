# Unlock an existing pasted age ID directly

## Objective and initial state

Fix the existing-ID onboarding path, which sent pasted IDs through the new-ID copy and backup dialog and could look as if Rawrarr still expected ID creation. Related context: [remember the age ID locally](2026-10-18-remember-age-id-local.md) and [age identity onboarding](2026-10-17-rawrarr-age-identity-onboarding.md).

## Actions

- Added an explicit **Unlock pasted ID** action that submits the pasted key directly for validation and vault unlock.
- Kept the copy and backup acknowledgement dialog for newly generated IDs.
- Kept the optional local remember checkbox available for either path and clarified that this flow accepts a post-quantum identity beginning `AGE-SECRET-KEY-PQ-1`.
- Added a regression assertion for pasted-ID direct unlock.

## Verification

Pending.

## Final state and remaining work

Pending.
