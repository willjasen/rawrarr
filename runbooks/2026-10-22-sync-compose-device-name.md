# Sync subscriber Compose device names

## Objective and initial state

Keep the coordinator's Devices name in sync with `RAWARR_DEVICE_NAME` for an already-approved node. The Mac test Compose specifies `MacBook Pro test1`, but the coordinator still shows `My fossil node`. The node keeps its approved bearer token in `/data/node-token`, so it registers as the existing node and does not submit a new enrollment request.

## Changes

- Include the configured public display name in node registration and each heartbeat registration.
- On the coordinator, accept the name as optional for compatibility with existing node binaries, validate it as 1–64 printable characters, and update only the approved enrollment row belonging to the authenticated node ID.
- The Devices view already reads that approved enrollment display name; the name updates on the node's next registration heartbeat after both binaries are updated.

## Verification

- Seven focused Node checks passed. The Rust release build in the Dockerfile completed successfully; the local Homebrew `cargo fmt --check` command could not start because its LLVM dependency requires a missing Z3 dylib.
- Rebuilt the isolated `rawrarr-orbstack-test` coordinator image and the Mac subscriber image, then recreated only the test `roarlay` and `fossil` containers with their named volumes attached. The HTTPS coordinator health route returns HTTP 200.
- The authenticated Devices page now shows a pending enrollment named `MacBook Pro test1`, matching `/Users/willjasen/rawrarr-test/compose.yml`. Its existing connected row (`device-5425…`) remains at the old name and is offline because the current Compose file requests a different node ID (`device-4c53353ce7ee06da0e220724`).
- The new pending row is deliberately awaiting owner approval. Its post-approval heartbeat name refresh has compiled but cannot be confirmed in the live connected-node list until the owner approves it.

## Recovery

Revert the registration payload and coordinator name update together, then rebuild the affected services. Existing node tokens and app data do not need to be replaced.
