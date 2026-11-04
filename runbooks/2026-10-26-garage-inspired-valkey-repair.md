# Garage-inspired Valkey mirror and repair design

## Objective and tracking

Record the original broader proposal for Valkey metadata mirroring and peer repair under issue [#100](https://github.com/willjasen/rawrarr/issues/100). The initial implementation later narrowed to encrypted `config.aged` repair; this runbook preserves the broader direction as historical context, not the current runtime contract.

## Current scope (updated 2026-11-03)

The merged implementation covers only encrypted `/shared/config.aged` repair. App metadata and artwork remain on their existing Syncthing path, and each node uses an independent Valkey mirror. The current protocol does not select a cluster-wide Valkey primary or implement automatic failover. See [the peer-repair protocol](../docs/metadata-peer-repair-protocol.md) and [the shared-cluster plan](../docs/shared-cluster-sharing-plan.md). Broader metadata-object and logical-mapping replication remains planned.

## Initial state

At the time of this design, existing docs described signed metadata revisions, user-controlled membership, RPC authentication, versioned logical/object Merkle views, deterministic conflict ordering, tombstone acknowledgements, and a disposable local Valkey cache. The initial peer-repair target was broader than the two-fossil implementation that was later merged.

## Decisions recorded

- `/shared` encrypted files remain authoritative; Valkey is a disposable, rebuildable mirror/cache.
- The original proposal covered all age-encrypted metadata objects and versioned logical mappings; that broader scope remains planned.
- Allow an approved device to author locally signed revisions while offline. They remain pending peer convergence and merge deterministically after reconnect; quorum does not block local authoring.
- Use periodic Merkle anti-entropy and device-key-authenticated RPC to find and repair gaps. Valkey is an independent per-node mirror, not a cluster-wide primary or the correctness protocol.
- The initial implementation covers only encrypted `/shared/config.aged`; app metadata and artwork remain on their existing Syncthing path. Its current protocol and verification status are documented in [the peer-repair protocol](../docs/metadata-peer-repair-protocol.md).
- Defer any automatic failover design. The current two-node implementation does not promote a primary or claim RF=3; any future failover requires a separately reviewed voter and fencing model.
- Garage uses periodic table resync with Merkle reconciliation; its durability repair guide also distinguishes metadata repair from data-block scrubbing. Valkey replication is asynchronous, so Rawrarr must rebuild mirror state from `/shared` and reconcile missing mirror entries.

## Remaining protocol details

Before expanding beyond config repair, specify metadata object/mapping canonicalization, how concurrent revisions and tombstones converge, schema negotiation, and bounded transfer recovery. Do not infer a cluster-wide primary or automatic failover from the independent Valkey mirrors.

## Verification

- Read the current design plan and prior decision runbook.
- The original design recorded issue #100 decisions and deferred a third node; the later implementation narrowed the first phase to config repair.
- Compared the design against Garage's [durability and repair guide](https://garagehq.deuxfleurs.fr/documentation/operations/durability-repairs/), [cluster deployment guide](https://garagehq.deuxfleurs.fr/documentation/cookbook/real-world/), and Valkey's [Sentinel documentation](https://valkey.io/topics/sentinel/).
- No application tests or live deployment were performed; no runtime behavior changed.

## Status and next step

The [shared-cluster plan](../docs/shared-cluster-sharing-plan.md) distinguishes the implemented config-only repair path from broader planned metadata synchronization. The [peer-repair protocol](../docs/metadata-peer-repair-protocol.md) is authoritative for the current config scope. Broader replication and any failover model require separate design and verification before implementation.
