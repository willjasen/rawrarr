# Fix AgeVault event-root build errors

Issue: [#486](https://github.com/willjasen/rawrarr/issues/486)

## Objective and initial state

Restore the `AgeVault::event_dir` API still used by authorized fossil RPC and restore compatibility migration helpers whose tests remained after the app-key changes. Main was at `9367aed`; builds failed because the API was missing, and the Rust test target also failed to compile on removed helpers.

## Changes

- Restored `AgeVault::event_dir`, preferring `RAWARR_EVT_DIR` and retaining `RAWARR_EVENT_DIR` as a compatibility alias; default is the `evt` sibling of shared data.
- Restored migration helpers for missing legacy metadata and ciphertext files into the selected fossil directory. Copies are bounded to regular files/trees, checksum verified where applicable, and never overwrite existing destinations or remove sources.
- Kept legacy config read/write selection unchanged.

## Verification

Using an isolated amd64 Docker clone from rebuilt `rawrarr-testing-base:local`, with current locked dependencies and networking disconnected during tests:

- `cargo test --locked --offline --manifest-path /workspace/rust-arr/Cargo.toml secure_config:: --no-fail-fast` — 41 passed, including the new event-root helper test.
- `cargo test --locked --offline --manifest-path /workspace/rust-arr/Cargo.toml config_repair --no-fail-fast` — 15 passed, including the two-node RPC conflict/reconnect/tombstone test.
- `cargo build --locked --offline --manifest-path /workspace/rust-arr/Cargo.toml` — passed on final source.
- `git diff --check` — passed.

The tests emit existing dead-code warnings. No live services were changed.
