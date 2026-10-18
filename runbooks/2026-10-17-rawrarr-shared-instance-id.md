# Rawrarr shared instance identity

## Objective and initial state

Record the user's requirement for one global, shared ID that identifies a particular Rawrarr folder across all nodes syncing it, distinct from each node's device ID. See [device architecture](../docs/device-architecture.md), [product direction](../docs/product-direction.md), and the [blind relay direction](2026-10-17-rawrarr-blind-relay-direction.md).

## Identity model recorded

- `instance_id` identifies one logical shared folder/namespace. Every participating node stores the same value.
- `device_id` identifies one node. It remains different even when multiple devices join the same shared instance; one device may join multiple instances.
- Syncthing's folder ID is a transport-specific mapping, not Rawrarr's `instance_id`, so another sync provider can join the same logical instance later.
- IDs are identifiers, not secrets or authorization. Authenticated device membership and user/share grants control access.
- A copied, independent fork gets a new `instance_id`; a replacement or additional node for the same shared folder preserves the current one.
- The relay may route by `instance_id` to member device IDs but still cannot authorize access or read payloads.

## Verification and remaining work

Updated product and device architecture documents. `git diff --check` and relative-link checks passed. This is a data-model decision only; no IDs, schemas, invitations, or relay routes have been implemented yet. Later implementation should define ID generation, the shared manifest/bootstrap flow, duplicate-ID detection, and database constraints.
