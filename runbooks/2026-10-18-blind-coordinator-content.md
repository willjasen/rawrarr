# Blind coordinator content boundary

## Objective and initial state

Record the requirement that the coordinator never handles plaintext subscriber metadata, including movie titles and TMDb IDs. The checkout contains extensive pre-existing changes. Existing design already requires browser-side age decryption for metadata export, but the current workspace relays node pages and can pass plaintext HTML/API responses through the coordinator.

## Actions

- Reviewed coordinator relay implementation, metadata export API/browser bundle, and current deployment/data-domain documentation.
- Added [blind coordinator content flow](../docs/blind-coordinator-content.md), defining ciphertext uploads from metadata/storage nodes, opaque coordinator inventory/routing, browser-side decryption, migration gap, and acceptance criteria.
- Updated the documentation and runbook indexes without changing mixed implementation files.

## Verification

Pending documentation consistency and whitespace checks. No runtime behavior changed; current node-page relay remains capable of carrying plaintext and does not meet the requested privacy boundary.

## Final state and remaining work

The target architecture and current limitation are documented. Implement ciphertext publication/storage or forwarding and a browser renderer as a follow-up implementation task. No commit was made because task-only additions coexist with extensive pre-existing changes and the project instructions prohibit a commit that would leave no uncommitted changes.
