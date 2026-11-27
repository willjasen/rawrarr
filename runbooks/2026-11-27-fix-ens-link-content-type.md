# Fix ENS wallet linking HTTP 415

Issue: https://github.com/willjasen/rawrarr/issues/484

## Objective and findings
Settings wallet linking failed with HTTP 415 after public audience configuration was restored (see [the preceding repair](2026-11-27-fix-public-ens-audience.md)). Both link requests JSON-stringified their bodies but omitted Content-Type. Axum's JSON extractor requires application/json. The shared request helper only adds Accept.

## Changes and adoption
Added explicit application/json headers to both wallet-link start and finish. Existing server authentication and signature validation remain unchanged. No migration required. Existing deployments adopt via the published application image after merge; browser reload loads the updated workspace script.

## Verification
Focused passkey UI test includes header regressions for both linking phases. Live signature flow remains unverified. No deployment performed for this change.

## Recovery
Deploy the prior published application image to roll back. No persistent data changes are introduced by this fix.
