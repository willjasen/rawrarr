# Stop a storage scan

## Objective

Let the owner stop a long-running storage-node media scan from the Devices view.

## Initial state

The UI could start and monitor scans, while the worker ran to completion. Hashing and probing are synchronous per media file, so interruption in the middle of a file could leave incomplete work.

## Actions

- Add a storage-node cancellation flag and protected cancel route, available to owner requests through direct and coordinator-relay paths.
- Check cancellation between files. The active file finishes hashing, probing, and encrypted sidecar publication before the worker exits.
- Add a Stop scan button, stopping status, and a terminal cancelled message with completed-file totals.
- Document the route and cancellation behavior in the device architecture.

## Verification

- `docker run --rm -v "$PWD/rust-arr:/src" -w /src rawrarr-testing-base:local cargo check --locked --offline` passed; it reported one pre-existing dead-code warning in `secure_config.rs`.
- `node --check rust-arr/src/coordinator-workspace.mjs` and `git diff --check` passed.
- Live deployment and cancellation verification require renewed temporary SSH authorization to the Mac mini. The prior temporary authorization was for the earlier progress task.

## Final state

Implementation is prepared locally. Neither Roarlay nor the Mac mini was deployed, so the UI and API are not yet live. Deployment and live cancellation verification remain outstanding until temporary Mini access is reauthorized.
