# Mixed and distributed media libraries

## Objective and initial state

Clarify how storage nodes represent libraries when one machine holds several media kinds or when media kinds live on different machines. Existing architecture already allowed multiple named libraries per device, but did not explicitly describe mixed roots, category-specific roots, or the combined catalog across nodes. The current scanner prototype still configures one local root with `RAWARR_LIBRARY_PATH`.

## Design decision

- A storage node may publish one mixed-media library or multiple named libraries backed by local roots under `/media`.
- A library's scanned inventory can contain multiple media kinds; the physical node is not limited to a single category.
- Users may distribute libraries across storage nodes, for example movies on one and television on another. The instance catalog combines these inventories, keyed by device, library, and relative path.
- Paths and media bytes remain local to each storage node. Shard and service nodes do not gain access to `/media` by virtue of catalog membership.
- Multi-library configuration and cross-node aggregation remain planned; the current local prototype has one configured library root.

## Changes and verification

Updated the product design, device architecture, node-role guidance, data-domain notes, and roadmap to record this behavior and its implementation status. Reviewed the documentation diff and ran `git diff --check`. No application code or runtime configuration changed.

## Final state and remaining work

The product model now supports both mixed-media libraries and category-specific libraries spread across storage nodes. Future implementation still needs durable library definitions, multi-root scanning, scan generations/reconciliation, and a combined catalog view.
