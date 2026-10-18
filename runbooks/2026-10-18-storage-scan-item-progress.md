# Report storage scan item progress

## Objective

Show completed media-file counts, such as “1 of 3”, while a storage-node scan runs.

## Initial state

- The Devices page showed an indeterminate progress bar and “Storage scan is running…” but the status API did not report incremental item counts.
- The worker already processes each selected media file sequentially and provides a bounded `maxFiles` option.

## Actions

- Add `filesCompleted` and `filesTotal` to storage scan status. Update those fields after each file is skipped, fails hashing/probing, or completes sidecar creation.
- Display the live count with the existing running message while keeping the progress bar indeterminate during media-tree enumeration and file processing.
- Update the architecture documentation and this task runbook.

## Verification

- `docker run --rm -v "$PWD/rust-arr:/src" -w /src rawrarr-testing-base:local cargo check --locked --offline` passed; `node --check rust-arr/src/coordinator-workspace.mjs` and `git diff --check` passed.
- `cargo fmt --check` could not run because `rustfmt` is not installed in the available toolchain.
- Rebuilt only Roarlay and only the Mini storage service. Roarlay health and its public tailnet route passed. The Mini is healthy, its scan-status route is present and owner-protected (HTTP 401 without a session), its direct health responds HTTP 204, and its Serve mapping remains `/direct-api` to port 8083.
- On the signed-in Devices page, started the bounded scan and confirmed “Storage scan is running… 0 of 3” alongside the progress bar. The scan then completed with 3 checked, 0 written, 3 already complete, and 0 errors. Since those files were already cached, the live request completed too quickly to observe the intermediate 1-of-3 and 2-of-3 states; the worker now emits those counts after each file completes during an uncached scan.

## Final state

- Storage scan status exposes per-file completion counts and the Devices view includes the current count with the running message. Both deployments are healthy and their deployment locks have been released. Temporary Mini SSH access was used for this task and no connection configuration was persisted.
