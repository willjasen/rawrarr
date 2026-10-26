# Garage-inspired Valkey mirror and repair design

## Objective and tracking

Record the owner's clarified architecture for Valkey metadata mirroring and peer repair under issue [#100](https://github.com/willjasen/rawrarr/issues/100). This is design work only; the internal Valkey service remains an artwork cache and no peer metadata RPC/Merkle implementation is present.

## Initial state

Started from latest fetched `origin/main` at `f378c9e`. Existing docs already describe signed metadata revisions, user-controlled membership, RPC authentication, versioned logical/object Merkle views, deterministic conflict ordering, tombstone acknowledgements, and a disposable local Valkey cache. The internal MacBook Valkey deployment is a local artwork cache. The user plans to add a fossil on the Mac mini; it is not yet available for two-node testing.

## Decisions recorded

- `/shared` encrypted files remain authoritative; Valkey is a disposable, rebuildable mirror/cache.
- Mirror all age-encrypted metadata objects and versioned logical mappings.
- Dynamically choose the writable Valkey primary and require safe-majority failover. A minority partition must not promote itself or become a second writer.
- Allow an approved device to author locally signed revisions while offline. They remain pending peer convergence and merge deterministically after reconnect; quorum does not block local authoring.
- Use periodic Merkle anti-entropy and device-key-authenticated RPC to find and repair gaps. Valkey primary/replica mirroring is an acceleration layer and not the correctness protocol.
- Defer a third node for the initial phase. A two-node MacBook/Mini setup can test convergence and repair after reconnect, but safe automatic failover is unavailable during a one-node partition. A third independent voter is needed before claiming automatic failover availability; a third data-bearing fossil is needed before claiming RF=3.
- Garage uses periodic table resync with Merkle reconciliation; its durability repair guide also distinguishes metadata repair from data-block scrubbing. Valkey replication is asynchronous, so Rawrarr must rebuild mirror state from `/shared` and reconcile missing mirror entries.

## Remaining protocol details

Specify how pending revisions are represented, how concurrent revision files pass through `/shared`/Syncthing, and how safe primary promotion is fenced. The initial two-node phase must prefer loss of failover availability over dual primaries; a third independent voter and a third data-bearing fossil remain deferred.

## Verification

- Read the current design plan and prior decision runbook.
- Confirmed issue #100 remains open and appended these design decisions there.
- Recorded the owner's decision to defer a third node in issue #100; the design now limits the first two-node phase to convergence/reconnect-repair validation.
- Compared the design against Garage's [durability and repair guide](https://garagehq.deuxfleurs.fr/documentation/operations/durability-repairs/), [cluster deployment guide](https://garagehq.deuxfleurs.fr/documentation/cookbook/real-world/), and Valkey's [Sentinel documentation](https://valkey.io/topics/sentinel/).
- No application tests or live deployment were performed; no runtime behavior changed.

## Status and next step

This runbook and [the shared-cluster plan](../docs/shared-cluster-sharing-plan.md) record design intent only. Specify pending-write and fencing behavior, then add sanitized RPC/Merkle/reconnect tests for the two-node phase before changing the MacBook runtime or claiming peer replication works. Keep failover unavailable until a third voter is added; keep RF=3 claims deferred until a third data-bearing fossil is added.
