# Render device pictures as fossils

## Objective and initial state

Change generated pictures on coordinator Devices cards from dinosaurs to fossils while keeping the account profile dinosaur unchanged. Work was isolated from a shared checkout with extensive staged and unstaged changes. The committed implementation derived a stable per-node seed from its public age recipient, then sent device cards to the same dinosaur renderer used by the profile.

GitHub issue lookup and creation were unavailable (search returned HTTP 422 and creation returned HTTP 404). Proposed issue:

> **Render generated device pictures as fossils**
>
> Change the generated picture shown for enrolled devices from a dinosaur to a fossil, while preserving deterministic per-device identity and the existing image delivery path.
>
> Acceptance criteria: device pictures depict fossils; remain deterministic and distinct across nodes; leave user profile artwork unchanged; add focused regression coverage; document implementation and verification in a dated runbook.

## Implementation

- Added a deterministic ammonite fossil illustration and separate authenticated PNG/JPEG XL device-picture routes. Existing profile-picture routes and dinosaur artwork remain unchanged.
- Updated Devices cards to use the device-picture routes while keeping the existing public-recipient-derived identity seed.
- Added Rust and coordinator UI regression assertions.

## Verification

- Passed: `node --test --test-name-pattern='device pictures use the deterministic fossil renderer' scripts/tests/test_rawrarr_coordinator_workspace.mjs` (1 test).
- Passed: `node --check rust-arr/src/coordinator-workspace.mjs` and `git diff --check`.
- Passed against rebased `origin/main`: `cargo test --locked coordinator_avatar::tests::fossil_device_art_is_deterministic_and_uses_a_shell_fossil -- --exact` in a Debian Rust container (1/1). Two unrelated compiler warnings were reported.
- A broader selection also ran the existing profile-renderer test, which currently fails on a stale loading-copy assertion (`metadata node` expected while the source says `fossil node`); the new isolated device-picture test passes.
- The host `cargo`/`rustfmt` toolchain cannot run because local LLVM requires a missing `libz3.5.1.dylib`. Rust formatting was reviewed manually; the container image did not include rustfmt.

## Final state and remaining work

The implementation is committed as `dc70422` on `codex/fossil-device-picture`, rebased onto `origin/main`. The branch has no upstream and remains local. No deployment was made: the coordinator UI deployment procedure requires a clean, published `main` revision, and this task could not create its required GitHub issue (search returned HTTP 422; creation returned HTTP 404). Deploy only `roarlay` after the change is published, then verify app health and the existing tailnet HTTPS route.
