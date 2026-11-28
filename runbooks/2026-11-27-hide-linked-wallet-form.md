# Hide linked wallet controls

Issue: https://github.com/willjasen/rawrarr/issues/502

## Objective
Hide the redundant ENS linking form after account wallet data confirms a linked wallet. Context: [wallet link content type repair](2026-11-27-fix-ens-link-content-type.md).

## Changes
Profile loading toggles the complete wallet linking form based on returned wallet count. The linked wallet status stays visible. An explicit hidden CSS rule overrides the form display styling. Existing post-link profile refresh applies the same behavior immediately. No data or protocol migration required; adoption requires the updated published application image and page reload.

## Verification
Focused Settings Playwright test in rawrarr-testing-base:local passed at 1280×800 and 375×812, checking linked/unlinked control visibility and horizontal overflow. Initial test caught the form CSS overriding hidden; explicit hidden styling resolved it. No live deployment or wallet authentication performed.

## Recovery
Revert the UI change in a later published image; no data rollback needed.
