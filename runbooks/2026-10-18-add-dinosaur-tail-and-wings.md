# Add variable tails and optional wings to profile dinosaurs

## Objective and initial state

Expand the public-age-ID-derived dinosaur avatar with short/long tail variants and an optional wing trait. Existing silhouettes had one fixed tail shape and no wings.

## Actions

- Derived tail reach and wing presence from the same identity digest used by the avatar's other traits. Some generated dinosaurs now have a folded wing; tail silhouette and length vary deterministically.
- Added regression coverage for tail variation, optional wings, and same-seed repeatability. Updated the design and script test indexes.

## Verification

- `node --test --test-name-pattern='dinosaur' scripts/tests/test_rawrarr_coordinator_workspace.mjs`: 3 passed. `node --check rust-arr/src/coordinator-workspace.mjs` and `git diff --check` passed. Tests cover repeated-seed stability, varied tail paths, optional wings, and the existing spike variants.
- Confirmed host port 7297 was unoccupied and 7298 belonged to the existing OrbStack Tailscale namespace. Rebuilt only `roarlay` in `rawrarr-local` with the coordinator and MacBook Tailscale Compose overlays.
- `rawrarr-local-roarlay-1` reports healthy; local and tailnet `/health` both returned HTTP 200. The deployed workspace module contains the tail-length generator. Tailscale Serve still routes `/` to `http://127.0.0.1:8080` and preserves `/direct-api` at `http://127.0.0.1:8082`.

## Final state and remaining work

- The same public-age-ID digest now deterministically selects a short or long tail and whether the dinosaur has folded wings.
