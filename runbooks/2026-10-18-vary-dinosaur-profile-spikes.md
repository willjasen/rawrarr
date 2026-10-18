# Vary dinosaur profile spikes

## Objective and initial state

Expand the public-age-ID-derived dinosaur avatar so back spikes are optional and can number zero through three. Existing previews all showed three spikes.

## Actions

- Made spike count, placement, width, and height deterministic traits from the avatar digest. One spike uses a larger variable silhouette; two spikes use visibly different heights and varied spacing.
- Added focused generation checks for all four counts, two-spike height differences, single-spike size variation, and repeatability. Updated the script test index.

## Verification

- `node --test scripts/tests/test_rawrarr_coordinator_workspace.mjs`: 12 passed, including seed repeatability, all four spike counts, multiple one-spike sizes, and unequal two-spike heights. `node --check rust-arr/src/coordinator-workspace.mjs` and `git diff --check` passed.
- Confirmed localhost port 7297 was unoccupied and port 7298 belonged to the existing OrbStack Tailscale namespace. Rebuilt only `roarlay` in the `rawrarr-local` Compose project using the coordinator and MacBook Tailscale overlays.
- `rawrarr-local-roarlay-1` reports healthy. Local and tailnet `/health` both returned HTTP 200. The deployed workspace module contains the 0–3 spike count logic. Tailscale Serve still routes `/` to `http://127.0.0.1:8080` and retains `/direct-api` at `http://127.0.0.1:8082`.

## Final state and remaining work

- Profile dinosaurs now have zero to three identity-derived back spikes. Single spikes are larger and vary in height; two-spike variants use different heights and two spacing patterns.
