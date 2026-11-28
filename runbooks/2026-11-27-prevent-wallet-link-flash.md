# Prevent wallet linking form flash

Issue: https://github.com/willjasen/rawrarr/issues/524

## Objective and initial state
The [linked wallet visibility fix](2026-11-27-hide-linked-wallet-form.md) hid the form only after an asynchronous account lookup, allowing a flash during refresh.

## Actions
The HTML now starts the form hidden. Existing profile loading reveals it only when wallet data reports no linked wallet. Failed or pending loads leave it hidden. No migration or backend changes; existing deployments adopt via the updated application image and browser reload.

## Verification
Focused Settings Playwright check passed in the isolated rawrarr-testing-base:local image at desktop 1280×800 and mobile 375×812. Checks cover initial hidden input/button, linked and unlinked visibility, and horizontal overflow. No live deployment performed.

## Recovery
Revert the HTML attribute in a later published image; no data rollback needed.
