# Retire the separate UI bundle updater

## Objective and initial state

Make published Rawrarr application images the only distribution path for UI assets. GitHub Actions already publishes `linux/amd64` and `linux/arm64` images for changes under `rust-arr/**`; however, a separate workflow also published UI release ZIPs, and services polled those releases into `/ui/current`. Runtime loading preferred those mounted files over the image's embedded assets. Issue [#436](https://github.com/willjasen/rawrarr/issues/436) tracks the change.

## Changes

- UI asset loading now always returns compile-time embedded assets, even when legacy `RAWARR_UI_DIR` settings or mounted `/ui/current` bundles exist.
- Removed the coordinator startup poller, updater program, UI release packaging/publishing workflow, publisher key, token wiring, and updater-only Compose overlay.
- Kept existing `/ui` volume mounts and volume declarations during rollout. The application ignores them, and this change does not delete their data.
- Documented that UI changes ship in the corresponding multi-architecture application image. No persistent data or protocol migration is required.

## Verification

- `python3 scripts/tests/test_image_ui_authority.py` — passed (4 tests).
- `python3 scripts/tests/test_internal_deploy_targets.py` — passed (10 tests).
- `node --test scripts/tests/test_rawrarr_internal_do3_compose.mjs scripts/tests/test_coordinator_availability.mjs scripts/tests/test_rawrarr_subscriber_compose.mjs` — passed (11 tests).
- `docker compose -f <file> config --no-interpolate --quiet` — passed for the root, coordinator, metadata, storage, and internal do3 Compose definitions.
- `git diff --check` — passed.
- `test_rawrarr_favicon.mjs` could not start because Playwright is not installed in this host checkout; its documented isolated Playwright environment was not available in this pass.
- Rust formatting/checking could not run: the local LLVM toolchain cannot load `libz3.5.1.dylib`, preventing `cargo fmt` and standalone `rustfmt` from starting. GitHub's image build will compile the changed Rust module after merge; this local compile remains unverified.

## Rollback and adoption

Deploy the new application image to adopt embedded-UI behavior. Old images can still use their preserved `/ui` contents. To roll back, restore the previous application image and its prior Compose configuration; no volume restoration is needed because this change leaves the volumes intact. UI-only publication no longer exists: edits under `rust-arr/` go through the image workflow and its deployment path.
