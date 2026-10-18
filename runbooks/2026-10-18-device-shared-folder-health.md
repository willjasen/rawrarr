# Device shared-folder health

## Objective
Show Syncthing shared-folder replication health in the Devices panel.

## Initial state
The local `/api/devices` endpoint listed only the current Rawrarr device. Syncthing status exposed daemon reachability but not folder or peer state. The checkout contained extensive unrelated staged/unstaged user work; this task edits only `rust-arr/src/syncthing.rs`, `rust-arr/src/main.rs`, and `rust-arr/src/ui.html` plus this runbook and its index entry.

## Changes
Added Syncthing REST reads for configured folders, local folder state, peer connectivity, and per-peer completion. `/api/devices` now returns the local Rawrarr device plus configured Syncthing peers, each with per-folder health. The Devices panel renders synced, syncing, disconnected, and unknown status distinctly.

## Verification
The Devices panel JavaScript syntax check (`node --check`) and focused `git diff --check` pass. `cargo check --manifest-path rust-arr/Cargo.toml` could not start because the installed `rustc` aborts while loading LLVM, which references a missing `libz3.5.1.dylib`.

## Final state
Implementation and UI checks are complete. Rust compilation remains unverified until the host toolchain dependency is repaired.
