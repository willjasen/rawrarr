# Add a rare golden dinosaur profile trait

## Objective and initial state

Add a rare visual trait to the deterministic dinosaur profile generator while continuing to derive every avatar from the account's public age ID. Existing identities already had deterministic species, palette, crest, spots, spikes, tail length, and optional wings.

## Actions

- Used an otherwise unused byte of the existing SHA-256 digest to select the golden palette at roughly 2% probability, preserving existing feature selections for all identities.
- Added warm gold coloring and small sparkle accents; ordinary palettes remain unchanged.
- Added deterministic rarity and palette coverage, and updated the design and test indexes.

## Verification

- `node --test --test-name-pattern='dinosaur' scripts/tests/test_rawrarr_coordinator_workspace.mjs`: 4 passed, covering the public-ID avatar, spike variation, tail/wings, and rare stable gold palette.
- `node --check rust-arr/src/coordinator-workspace.mjs` and `git diff --check` passed.
- Confirmed port 7297 was unoccupied and 7298 belonged to the existing OrbStack listener. After waiting for the existing roarlay deployment lock to clear, rebuilt only `roarlay` in `rawrarr-local` with the documented coordinator and MacBook Tailscale Compose overlays.
- `rawrarr-local-roarlay-1` reports healthy; local and tailnet `/health` returned HTTP 200. The deployed workspace asset contains the gold palette. Tailscale Serve routes `/` to `http://127.0.0.1:8080` and retains `/direct-api` at `http://127.0.0.1:8082`.

## Final state and remaining work

- A small, deterministic fraction of public-age-ID-derived dinosaur avatars now use a golden palette with sparkle accents. No image or avatar seed is stored server-side.
