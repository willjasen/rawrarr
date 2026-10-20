# Plan fossil and boneyard node terminology

## Decision

Use **fossil** as the future user-facing name for existing storage/apex media-source nodes. Use **boneyard** for a future untrusted Syncthing device that receives selected media from a fossil and stores the received files encrypted at rest by Syncthing.

## Scope

This records terminology and future direction only. Existing `apex` Compose services and containers, the internal `storage` role, and current trusted storage-to-storage synchronization remain unchanged. The untrusted boneyard media-sharing feature is not implemented. Syncthing transport encryption alone does not encrypt replicated files at rest.

## Documentation updated

- Added the planned terminology and role distinction to [node types](../docs/node-types.md).
- Added the product-design distinction between current trusted replication and future boneyard storage to [product and interface design](../docs/DESIGN.md).
- Added the terminology migration and future feature to the [roadmap](../ROADMAP.md).

## Verification

Documentation-only change; no runtime behavior or deployment was changed.
