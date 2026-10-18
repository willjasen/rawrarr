# Consolidate Rawrarr project status into the roadmap

## Objective

Create one project roadmap covering completed and outstanding work, and replace duplicate implementation-status lists elsewhere with links to it.

## Initial state

`docs/technical-status.md` contained separate implemented and planned lists. `docs/product-direction.md`, `docs/device-architecture.md`, `docs/user-guide.md`, and the root README repeated current status or next-work queues. Preserve architecture constraints, user procedures, and product intent in their own docs while centralizing the completion state and recommended sequence.

## Changes

- Added root `ROADMAP.md` with completed capabilities, prioritized next work, and later product/operations work, without delivery dates.
- Replaced duplicated status lists in the technical-status, product-direction, device-architecture, and user-guide docs with roadmap links.
- Simplified the root README and linked the roadmap from the documentation index and product design guide.
- Preserved the detailed usage flow, architectural trust boundaries, and product direction in their existing docs.

## Verification

- Local Markdown links in 10 updated documentation files resolve.
- `git diff --check` passed.

## Final state

`ROADMAP.md` is the single completed/outstanding work list and has the next recommended milestones. The root README and documentation index link to it; the technical-status, product-direction, device-architecture, and user-guide pages no longer repeat implementation queues. Design principles and architectural constraints remain in their respective docs.
