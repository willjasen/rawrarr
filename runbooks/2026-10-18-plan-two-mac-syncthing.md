# Plan shared-folder replication on both Macs

## Objective and initial state

Make Syncthing setup an explicit requirement of the MacBook coordinator/Mac mini storage deployment. Reviewed the [node-linking decisions](2026-10-18-coordinator-storage-device-linking.md), [original Mini plan](2026-10-18-mac-mini-storage-node-plan.md), current Compose inventory, user guide, and test index. Base Compose has no Syncthing service; an isolated OrbStack sync test exists. No live Syncthing or host state was assessed.

## Decisions and actions

Expanded the [deployment plan](../docs/mac-mini-storage-node-plan.md) with one Syncthing container per Mac, matching local `/shared` mounts, separate persistent Syncthing identities/config, explicit approved peer/folder mapping, and initial synchronization from the existing coordinator instance. Selected bidirectional replication for coordinator settings and worker metadata, with distinct application write ownership.

Documented private peer transport separately from the coordinator HTTPS endpoint, protected GUI/API access, permission checks, secret/media exclusions, deletion/conflict recovery, and removal of sync access on revocation. Shared logs are not all encrypted; human enrollment codes must stay out of them. Reviewed official Syncthing Docker/firewall guidance.

## Verification and final state

Checked documentation links and task-only whitespace/diff. No application tests apply to this documentation-only change. No containers were started, peers linked, files synchronized, or network settings changed. Existing unrelated changes remain preserved.

Next: implement persistent Compose packaging and approved peer provisioning, then verify bidirectional transfer, restart/reconnect behavior, exclusions, and catalog ingestion on the actual two-Mac deployment. Retain independent backups; pause only the new peer/folder relationship if setup fails.
