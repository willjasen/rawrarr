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

## Merge and deployment

- PR [#31](https://github.com/willjasen/rawrarr/pull/31) merged into `main` as `742f75d037ab1b26b27e6ae02a69d9c0ae7967cd`. GitHub Actions run [39863134542](https://github.com/willjasen/rawrarr/actions/runs/39863134542) passed and published UI release `ui-742f75d037ab1b26b27e6ae02a69d9c0ae7967cd` with bundle SHA-256 `b3ac9e4528ae0bf54539daa5440a57ae068b1e8eb0cb363d182adf7a7ca008b8`.
- The clean Mac mini `github-source` checkout was fast-forwarded from `9921d01` to the published merge commit. After confirming the roarlay deployment lock was free, this task atomically claimed it and rebuilt/recreated only `roarlay` in `rawrarr-internal-mini` with `--no-deps --build`. The initial build could not find the Docker keychain helper in the SSH shell PATH; retrying with `/usr/local/bin` in PATH succeeded. New image: `sha256:ee906f216a42de9e2ca52c6c5e8af363acb7e7e370a42f411996c33f021ce333`.
- Roarlay is healthy with zero restarts. PostgreSQL and the Tailscale sidecar remained running. Tailscale Serve read-back confirms HTTPS 443 `/` still proxies to `http://127.0.0.1:8080`; the HTTPS health page returned HTTP 200 and identifies roarlay. The installed UI bundle version and SHA-256 match the published release. The served workspace uses device-picture routes for nodes and profile-picture routes for the account; unauthenticated device-picture requests return HTTP 401.
- The roarlay deployment lock was removed and confirmed absent. The MacBook fossil service was not changed. An authenticated browser session was not used for visual review of the device pictures.

## Final state and remaining work

The fossil device-picture change is merged and deployed to the internal coordinator. GitHub issue lookup returned HTTP 422 and issue creation returned HTTP 404 through the available GitHub connector, so the proposed issue text remains recorded above. No runtime or subscriber-node follow-up is pending; visual confirmation in an authenticated browser remains unverified.
