# Document Rawrarr's unified product direction

## Objective and initial state

Record the user's direction that Rawrarr could replace Prowlarr's role while providing broader media automation. Existing documentation described an early Radarr-focused media library, with discovery/indexer management absent.

## Actions

- Added a product direction document covering discovery, library management, fulfillment, remote devices, and eventually consistent shared state.
- Clarified that existing Arr integrations remain supported during migration and that current Rawrarr capabilities are not Prowlarr parity.
- Linked the direction from the device architecture and Rust application README.

## Verification

- `git diff --check` passed.
- Relative links in the changed product, architecture, README, and runbook documents resolve.
- Reviewed the wording to distinguish intended product direction from the current Radarr-focused prototype.

## Final state and remaining work

The intended product boundary and current implementation limits are documented. Indexer management and cross-provider search remain future implementation work.
