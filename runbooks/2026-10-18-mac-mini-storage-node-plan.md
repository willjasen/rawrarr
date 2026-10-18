# Plan the Mac mini storage node

## Objective and initial state

Plan an OrbStack storage-node deployment on the Mac mini. The user confirmed `/Volumes/silo/Media/data` and joining the current instance. This is planning only; no SSH, installation, container changes, drive scans, or synchronization were performed.

The repository is on `main`, has no configured remote/upstream, and contains extensive pre-existing tracked and untracked changes. Preserve them and stage only this task's documentation hunks.

Reviewed the current Compose files, Dockerfile, scanner/device source, architecture, and prior [local scan](2026-10-17-local-library-scan.md), [mixed libraries](2026-10-18-mixed-and-distributed-media-libraries.md), and [container distribution](2026-10-18-rawrarr-container-distribution.md) runbooks.

## Actions and decisions

Created the [Mac mini deployment plan](../docs/mac-mini-storage-node-plan.md). Recorded a read-only media mount, separate local state, existing-instance bootstrap, localhost:7297 WebAuthn alignment, and phased acceptance. Checked official OrbStack and Docker documentation for bind mounts and missing-path behavior. Documented multi-node authentication/catalog and drive-loss handling as gaps requiring verification or implementation.

## Verification and final state

Reviewed task-only staged changes, checked local Markdown links, and ran `git diff --cached --check`. No application behavior changed, so application tests and service rebuilds are not applicable. The Mac mini, drive, OrbStack runtime, shared transport, and authentication flow remain unverified. No deployment or rollback was needed. Next step is a read-only host preflight followed by implementation of the documented bootstrap and mount safeguards.
