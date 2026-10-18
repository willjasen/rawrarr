# Improve dinosaur background scenes

## Objective and initial state

Improve forest tree visibility in the deterministic coordinator-generated dinosaur avatar and add additional scene options. Preserve public-age-ID determinism, authenticated server-side JPEG XL output, and the existing wing/spike rule.

Related runbooks: [Add dinosaur background scenes](2026-10-18-add-dinosaur-background-scenes.md) and [Serve dinosaur avatars as JPEG XL](2026-10-18-serve-dinosaur-avatars-as-jxl.md).

## Actions

- Repositioned forest trees to either side of the centered dinosaur and increased foliage contrast, leaving trunks visible.
- Added desert mesa, snowy mountains, and tropical palm scenes alongside volcano, forest, and hills/sunset.
- Expanded deterministic scene selection from three to six using the existing digest byte; did not add stored image data or alter avatar delivery formats.
- Updated the design, runbook index, and test index.

## Verification

- `RUSTC="$HOME/.rustup/toolchains/1.85.1-x86_64-apple-darwin/bin/rustc" ~/.cargo/bin/cargo test coordinator_avatar::tests --bin rust-arr` passed: 5 tests, including selection across all six scenes and both raster output formats.
- Rendered and inspected all six scenes using the backend generator and `rsvg-convert`; the forest now has clearly visible left/right trees. Temporary preview outputs were kept in `/tmp/rawrarr-avatar-scenes`.
- Rebuilt only `roarlay` in Compose project `rawrarr-local`. The container reports healthy; local `http://127.0.0.1:7298/health` and tailnet `/health` returned HTTP 200. Tailscale Serve still routes `/` to `http://127.0.0.1:8080` and retains `/direct-api` at `http://127.0.0.1:8082`.
- An anonymous GET to the JXL profile-picture endpoint returned HTTP 401, preserving the authenticated delivery boundary.
- The deployment lock was confirmed released. Compose emitted an existing orphan warning for `rawrarr-local-syncthing-1`; no orphan cleanup was requested or performed.
- `git diff --check` passed.

## Final state and remaining work

Roarlay now serves six public-ID-derived avatar scenes with better-separated forest trees. The coordinator continues to generate images on demand as JPEG XL with PNG fallback and stores no avatar image data. No remaining work for this change.
